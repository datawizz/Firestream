//! App lifecycle management

use crate::core::Result;
use crate::deploy::helm_lifecycle::{
    HelmChart, ChartInfo, ChartPayload, HelmAction, ValidationResult,
    HelmChartNamespace, ChartSetValue, CommonChart,
};
use super::{AppManifest, AppStructure, ManifestParser, BootstrapExecutor, AppBuilder};
use std::path::Path;
use tracing::{info, debug};
use async_trait::async_trait;

/// App lifecycle manager that implements HelmChart
pub struct AppLifecycle {
    /// App manifest
    pub manifest: AppManifest,
    /// App structure
    pub structure: AppStructure,
    /// Generated or loaded chart info
    pub chart_info: ChartInfo,
    /// Environment
    pub environment: Option<String>,
}

impl AppLifecycle {
    /// Create from app directory
    pub async fn from_directory(
        app_dir: impl AsRef<Path>,
        action: HelmAction,
        environment: Option<String>,
    ) -> Result<Self> {
        let app_dir = app_dir.as_ref();
        let structure = AppStructure::from_root(app_dir);
        
        // Validate structure
        structure.validate()?;
        
        // Load manifest with environment overrides
        let manifest = ManifestParser::load_with_environment(
            &structure.manifest,
            environment.as_deref(),
        ).await?;
        
        // Generate or load chart info
        let chart_info = Self::create_chart_info(&manifest, &structure, action)?;
        
        Ok(Self {
            manifest,
            structure,
            chart_info,
            environment,
        })
    }
    
    /// Create chart info from manifest
    fn create_chart_info(
        manifest: &AppManifest,
        structure: &AppStructure,
        action: HelmAction,
    ) -> Result<ChartInfo> {
        let namespace = manifest.deploy.namespace.clone()
            .unwrap_or_else(|| manifest.app.name.clone());
        
        let mut chart_info = if let Some(helm_config) = &manifest.deploy.helm {
            // Use external chart
            ChartInfo {
                name: manifest.app.name.clone(),
                repository: helm_config.repository.clone(),
                chart: helm_config.chart.clone().unwrap_or(manifest.app.name.clone()),
                version: helm_config.version.clone(),
                namespace: HelmChartNamespace::Firestream,
                custom_namespace: Some(namespace),
                action,
                ..Default::default()
            }
        } else {
            // Use generated chart
            ChartInfo {
                name: manifest.app.name.clone(),
                chart: format!("./{}", manifest.app.name),
                namespace: HelmChartNamespace::Firestream,
                custom_namespace: Some(namespace),
                action,
                ..Default::default()
            }
        };
        
        // Add values file if exists
        if let Some(values_path) = &structure.values {
            chart_info.values_files.push(values_path.clone());
        }
        
        // Convert config to Helm values
        let values = Self::config_to_helm_values(manifest);
        chart_info.values = values;
        
        Ok(chart_info)
    }
    
    /// Convert app config to Helm values
    fn config_to_helm_values(manifest: &AppManifest) -> Vec<ChartSetValue> {
        let mut values = Vec::new();
        
        // Image configuration
        if let Some(registry) = &manifest.build.registry {
            values.push(ChartSetValue {
                key: "image.registry".to_string(),
                value: registry.clone(),
            });
        }
        
        if let Some(repository) = &manifest.build.repository {
            values.push(ChartSetValue {
                key: "image.repository".to_string(),
                value: repository.clone(),
            });
        }
        
        let tag = manifest.build.tag.clone()
            .unwrap_or_else(|| manifest.app.version.clone());
        values.push(ChartSetValue {
            key: "image.tag".to_string(),
            value: tag,
        });
        
        // Resources
        if let Some(cpu_request) = &manifest.config.resources.cpu_request {
            values.push(ChartSetValue {
                key: "resources.requests.cpu".to_string(),
                value: cpu_request.clone(),
            });
        }
        
        if let Some(memory_request) = &manifest.config.resources.memory_request {
            values.push(ChartSetValue {
                key: "resources.requests.memory".to_string(),
                value: memory_request.clone(),
            });
        }
        
        if let Some(cpu_limit) = &manifest.config.resources.cpu_limit {
            values.push(ChartSetValue {
                key: "resources.limits.cpu".to_string(),
                value: cpu_limit.clone(),
            });
        }
        
        if let Some(memory_limit) = &manifest.config.resources.memory_limit {
            values.push(ChartSetValue {
                key: "resources.limits.memory".to_string(),
                value: memory_limit.clone(),
            });
        }
        
        // Scaling
        values.push(ChartSetValue {
            key: "replicaCount".to_string(),
            value: manifest.config.scaling.replicas.to_string(),
        });
        
        if manifest.config.scaling.min_replicas.is_some() || manifest.config.scaling.max_replicas.is_some() {
            values.push(ChartSetValue {
                key: "autoscaling.enabled".to_string(),
                value: "true".to_string(),
            });
            
            if let Some(min) = manifest.config.scaling.min_replicas {
                values.push(ChartSetValue {
                    key: "autoscaling.minReplicas".to_string(),
                    value: min.to_string(),
                });
            }
            
            if let Some(max) = manifest.config.scaling.max_replicas {
                values.push(ChartSetValue {
                    key: "autoscaling.maxReplicas".to_string(),
                    value: max.to_string(),
                });
            }
            
            if let Some(cpu) = manifest.config.scaling.target_cpu_utilization {
                values.push(ChartSetValue {
                    key: "autoscaling.targetCPUUtilizationPercentage".to_string(),
                    value: cpu.to_string(),
                });
            }
        }
        
        values
    }
    
    /// Build container image
    pub async fn build_image(&self) -> Result<String> {
        info!("Building container image for app '{}'", self.manifest.app.name);
        
        let builder = AppBuilder::new(&self.manifest, &self.structure);
        let image_tag = builder.build().await?;
        
        info!("Successfully built image: {}", image_tag);
        Ok(image_tag)
    }
    
    /// Generate Helm chart if needed
    pub async fn generate_chart(&self) -> Result<()> {
        if self.manifest.deploy.helm.is_some() {
            debug!("Using external Helm chart, skipping generation");
            return Ok(());
        }
        
        info!("Generating Helm chart for app '{}'", self.manifest.app.name);
        
        // Create chart directory
        let chart_dir = self.structure.root.join("chart");
        tokio::fs::create_dir_all(&chart_dir).await?;
        
        // Generate Chart.yaml
        let chart_yaml = format!(r#"apiVersion: v2
name: {}
description: {}
type: application
version: {}
appVersion: "{}"
"#, 
            self.manifest.app.name,
            self.manifest.app.description.as_deref().unwrap_or("A Firestream app"),
            self.manifest.app.version,
            self.manifest.app.version
        );
        
        tokio::fs::write(chart_dir.join("Chart.yaml"), chart_yaml).await?;
        
        // Generate templates
        self.generate_templates(&chart_dir).await?;
        
        // Generate values.yaml if not exists
        if !self.structure.values.is_some() {
            self.generate_values(&chart_dir).await?;
        }
        
        Ok(())
    }
    
    /// Generate Helm templates
    async fn generate_templates(&self, chart_dir: &Path) -> Result<()> {
        let templates_dir = chart_dir.join("templates");
        tokio::fs::create_dir_all(&templates_dir).await?;
        
        // Generate deployment.yaml
        let deployment = self.generate_deployment_template();
        tokio::fs::write(templates_dir.join("deployment.yaml"), deployment).await?;
        
        // Generate service.yaml if ports are defined
        if !self.manifest.config.ports.is_empty() {
            let service = self.generate_service_template();
            tokio::fs::write(templates_dir.join("service.yaml"), service).await?;
        }
        
        // Generate ingress.yaml if configured
        if let Some(ingress_config) = &self.manifest.deploy.ingress {
            let ingress = self.generate_ingress_template(ingress_config);
            tokio::fs::write(templates_dir.join("ingress.yaml"), ingress).await?;
        }
        
        Ok(())
    }
    
    /// Generate deployment template
    fn generate_deployment_template(&self) -> String {
        let container_name = self.manifest.app.name.replace('-', "");
        
        let mut template = format!(r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ include "{}.fullname" . }}
  labels:
    {{- include "{}.labels" . | nindent 4 }}
spec:
  {{- if not .Values.autoscaling.enabled }}
  replicas: {{ .Values.replicaCount }}
  {{- end }}
  selector:
    matchLabels:
      {{- include "{}.selectorLabels" . | nindent 6 }}
  template:
    metadata:
      labels:
        {{- include "{}.selectorLabels" . | nindent 8 }}
    spec:
      containers:
      - name: {}
        image: "{{ .Values.image.repository }}:{{ .Values.image.tag | default .Chart.AppVersion }}"
        imagePullPolicy: {{ .Values.image.pullPolicy }}
"#, 
            self.manifest.app.name,
            self.manifest.app.name,
            self.manifest.app.name,
            self.manifest.app.name,
            container_name
        );
        
        // Add ports
        if !self.manifest.config.ports.is_empty() {
            template.push_str("        ports:\n");
            for port in &self.manifest.config.ports {
                template.push_str(&format!(
                    "        - name: {}\n          containerPort: {}\n          protocol: {}\n",
                    port.name, port.port, port.protocol
                ));
            }
        }
        
        // Add environment variables
        if !self.manifest.config.env.is_empty() || !self.manifest.secrets.refs.is_empty() {
            template.push_str("        env:\n");
            
            for (key, value) in &self.manifest.config.env {
                template.push_str(&format!(
                    "        - name: {}\n          value: \"{}\"\n",
                    key, value
                ));
            }
            
            for secret_ref in &self.manifest.secrets.refs {
                template.push_str(&format!(
                    "        - name: {}\n          valueFrom:\n            secretKeyRef:\n              name: {}\n              key: {}\n",
                    secret_ref.env_var,
                    secret_ref.name,
                    secret_ref.key.as_deref().unwrap_or("value")
                ));
            }
        }
        
        // Add resource limits
        template.push_str("        resources:\n          {{- toYaml .Values.resources | nindent 10 }}\n");
        
        // Add probes
        if let Some(liveness) = &self.manifest.config.health.liveness {
            template.push_str(&self.generate_probe("livenessProbe", liveness));
        }
        
        if let Some(readiness) = &self.manifest.config.health.readiness {
            template.push_str(&self.generate_probe("readinessProbe", readiness));
        }
        
        template
    }
    
    /// Generate probe configuration
    fn generate_probe(&self, probe_type: &str, config: &super::schema::ProbeConfig) -> String {
        let mut probe = format!("        {}:\n", probe_type);
        
        match &config.probe_type {
            super::schema::ProbeType::Http { path, port } => {
                probe.push_str(&format!("          httpGet:\n            path: {}\n", path));
                if let Some(p) = port {
                    probe.push_str(&format!("            port: {}\n", p));
                } else if let Some(first_port) = self.manifest.config.ports.first() {
                    probe.push_str(&format!("            port: {}\n", first_port.name));
                }
            }
            super::schema::ProbeType::Tcp { port } => {
                probe.push_str(&format!("          tcpSocket:\n            port: {}\n", port));
            }
            super::schema::ProbeType::Exec { command } => {
                probe.push_str("          exec:\n            command:\n");
                for cmd in command {
                    probe.push_str(&format!("            - {}\n", cmd));
                }
            }
        }
        
        probe.push_str(&format!("          initialDelaySeconds: {}\n", config.initial_delay_seconds));
        probe.push_str(&format!("          periodSeconds: {}\n", config.period_seconds));
        probe.push_str(&format!("          timeoutSeconds: {}\n", config.timeout_seconds));
        probe.push_str(&format!("          successThreshold: {}\n", config.success_threshold));
        probe.push_str(&format!("          failureThreshold: {}\n", config.failure_threshold));
        
        probe
    }
    
    /// Generate service template
    fn generate_service_template(&self) -> String {
        let mut template = format!(r#"apiVersion: v1
kind: Service
metadata:
  name: {{ include "{}.fullname" . }}
  labels:
    {{- include "{}.labels" . | nindent 4 }}
spec:
  type: {{ .Values.service.type }}
  ports:
"#, 
            self.manifest.app.name,
            self.manifest.app.name
        );
        
        for port in &self.manifest.config.ports {
            let service_port = port.service_port.unwrap_or(port.port);
            template.push_str(&format!(
                "  - port: {}\n    targetPort: {}\n    protocol: {}\n    name: {}\n",
                service_port, port.name, port.protocol, port.name
            ));
        }
        
        template.push_str(&format!(
            "  selector:\n    {{- include \"{}.selectorLabels\" . | nindent 4 }}\n",
            self.manifest.app.name
        ));
        
        template
    }
    
    /// Generate ingress template
    fn generate_ingress_template(&self, config: &super::schema::IngressConfig) -> String {
        let mut template = format!(r#"{{- if .Values.ingress.enabled -}}
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: {{ include "{}.fullname" . }}
  labels:
    {{- include "{}.labels" . | nindent 4 }}
  {{- with .Values.ingress.annotations }}
  annotations:
    {{- toYaml . | nindent 4 }}
  {{- end }}
spec:
"#, 
            self.manifest.app.name,
            self.manifest.app.name
        );
        
        if let Some(class) = &config.class {
            template.push_str(&format!("  ingressClassName: {}\n", class));
        }
        
        if !config.tls.is_empty() {
            template.push_str("  tls:\n");
            for tls in &config.tls {
                template.push_str(&format!("  - hosts:\n"));
                for host in &tls.hosts {
                    template.push_str(&format!("    - {}\n", host));
                }
                template.push_str(&format!("    secretName: {}\n", tls.secret_name));
            }
        }
        
        template.push_str("  rules:\n");
        for host in &config.hosts {
            template.push_str(&format!("  - host: {}\n    http:\n      paths:\n", host.host));
            
            let paths = if host.paths.is_empty() {
                // Default path
                vec![super::schema::IngressPath {
                    path: "/".to_string(),
                    path_type: "Prefix".to_string(),
                    port: None,
                }]
            } else {
                host.paths.clone()
            };
            
            for path in paths {
                template.push_str(&format!(
                    "      - path: {}\n        pathType: {}\n        backend:\n          service:\n            name: {{ include \"{}.fullname\" . }}\n            port:\n              number: {}\n",
                    path.path,
                    path.path_type,
                    self.manifest.app.name,
                    path.port.unwrap_or_else(|| {
                        self.manifest.config.ports.first()
                            .map(|p| p.service_port.unwrap_or(p.port))
                            .unwrap_or(80)
                    })
                ));
            }
        }
        
        template.push_str("{{- end }}\n");
        template
    }
    
    /// Generate values.yaml
    async fn generate_values(&self, chart_dir: &Path) -> Result<()> {
        let values = format!(r#"# Default values for {}.
replicaCount: {}

image:
  repository: {}
  pullPolicy: IfNotPresent
  tag: ""

service:
  type: ClusterIP

ingress:
  enabled: false
  className: ""
  annotations: {{}}
  hosts: []
  tls: []

resources:
  limits:
    cpu: {}
    memory: {}
  requests:
    cpu: {}
    memory: {}

autoscaling:
  enabled: false
  minReplicas: 1
  maxReplicas: 100
  targetCPUUtilizationPercentage: 80
"#,
            self.manifest.app.name,
            self.manifest.config.scaling.replicas,
            self.manifest.build.repository.as_deref().unwrap_or(&self.manifest.app.name),
            self.manifest.config.resources.cpu_limit.as_deref().unwrap_or("1000m"),
            self.manifest.config.resources.memory_limit.as_deref().unwrap_or("512Mi"),
            self.manifest.config.resources.cpu_request.as_deref().unwrap_or("100m"),
            self.manifest.config.resources.memory_request.as_deref().unwrap_or("128Mi")
        );
        
        tokio::fs::write(chart_dir.join("values.yaml"), values).await?;
        Ok(())
    }
}

#[async_trait]
impl HelmChart for AppLifecycle {
    fn get_chart_info(&self) -> &ChartInfo {
        &self.chart_info
    }
    
    fn get_chart_info_mut(&mut self) -> &mut ChartInfo {
        &mut self.chart_info
    }
    
    async fn pre_exec(
        &self,
        _kubernetes_config: &Path,
        _envs: &[(String, String)],
        mut payload: Option<ChartPayload>,
    ) -> Result<Option<ChartPayload>> {
        info!("Pre-exec: Running bootstrap.sh for app '{}'", self.manifest.app.name);
        
        // Execute bootstrap script
        let executor = BootstrapExecutor::new(&self.structure.bootstrap);
        executor.execute(&self.structure.root, &self.manifest).await?;
        
        // Build container image if needed
        if self.chart_info.action == HelmAction::Deploy {
            let image_tag = self.build_image().await?;
            
            if let Some(ref mut p) = payload {
                p.insert("built_image", image_tag)?;
            }
            
            // Generate Helm chart if needed
            self.generate_chart().await?;
        }
        
        Ok(payload)
    }
    
    async fn exec(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
        payload: Option<ChartPayload>,
    ) -> Result<Option<ChartPayload>> {
        // Use CommonChart for actual deployment
        let common = CommonChart::new(self.chart_info.clone());
        common.exec(kubernetes_config, envs, payload).await
    }
    
    async fn post_exec(
        &self,
        _kubernetes_config: &Path,
        _envs: &[(String, String)],
        mut payload: Option<ChartPayload>,
    ) -> Result<Option<ChartPayload>> {
        if self.chart_info.action == HelmAction::Deploy {
            info!("Post-exec: App '{}' deployed successfully", self.manifest.app.name);
            
            // Store app metadata in payload
            if let Some(ref mut p) = payload {
                p.insert("app_name", &self.manifest.app.name)?;
                p.insert("app_version", &self.manifest.app.version)?;
                p.insert("app_type", format!("{:?}", self.manifest.app.app_type))?;
            }
        }
        
        Ok(payload)
    }
    
    async fn validate(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
        payload: Option<ChartPayload>,
    ) -> Result<Vec<ValidationResult>> {
        let common = CommonChart::new(self.chart_info.clone());
        let mut results = common.validate(kubernetes_config, envs, payload).await?;
        
        // Add app-specific validations
        if self.chart_info.action == HelmAction::Deploy {
            // Validate dependencies are deployed
            for (dep_name, _dep) in &self.manifest.dependencies {
                let dep_check = self.check_dependency_deployed(kubernetes_config, envs, dep_name).await?;
                results.push(dep_check);
            }
        }
        
        Ok(results)
    }
    
    async fn collect_events(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
    ) -> Result<Vec<crate::deploy::helm_lifecycle::KubernetesEvent>> {
        let common = CommonChart::new(self.chart_info.clone());
        common.collect_events(kubernetes_config, envs).await
    }
    
    async fn get_release_info(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
    ) -> Result<crate::deploy::helm_lifecycle::HelmReleaseStatus> {
        let common = CommonChart::new(self.chart_info.clone());
        common.get_release_info(kubernetes_config, envs).await
    }
    
    async fn is_deployed(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
    ) -> Result<bool> {
        let common = CommonChart::new(self.chart_info.clone());
        common.is_deployed(kubernetes_config, envs).await
    }
}

impl AppLifecycle {
    /// Check if a dependency is deployed
    async fn check_dependency_deployed(
        &self,
        kubernetes_config: &Path,
        envs: &[(String, String)],
        dep_name: &str,
    ) -> Result<ValidationResult> {
        use crate::deploy::helm_lifecycle::executor::is_helm_release_deployed;
        
        // Assume dependency is deployed in its own namespace
        let is_deployed = is_helm_release_deployed(
            kubernetes_config,
            envs,
            dep_name,
            dep_name,
        ).await?;
        
        Ok(ValidationResult {
            check_name: format!("dependency_{}", dep_name),
            passed: is_deployed,
            message: if is_deployed {
                format!("Dependency '{}' is deployed", dep_name)
            } else {
                format!("Dependency '{}' is not deployed", dep_name)
            },
            details: None,
        })
    }
}
