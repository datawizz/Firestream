export type StackKey = "microsoft" | "google" | "amazon" | "firestream";

export interface PlatformTier {
  // Inclusive upper bound. The largest tier uses Number.POSITIVE_INFINITY.
  maxProcessedGB: number;
  monthly: number;
  label: string;
}

export interface StackRates {
  key: StackKey;
  label: string;
  sublabel: string;
  storageRate: number;
  processingRate: number;
  seatRate: number;
  // Stepped platform/orchestration cost — auto-selected by processedGB.
  platformTiers: PlatformTier[];
  // Vendor's own reference-architecture article for this stack.
  recommendedStackUrl: string;
}

// USD, mid-2026, US regions, list/PAYG. Re-verify before any public release.
export const RATES: Record<StackKey, StackRates> = {
  microsoft: {
    key: "microsoft",
    label: "Microsoft",
    sublabel: "OneLake + Fabric + Power BI",
    storageRate: 0.018,
    processingRate: 0.0065,
    seatRate: 14.0,
    // Fabric F-SKU ladder. Each step roughly doubles capacity and price.
    platformTiers: [
      { maxProcessedGB: 10_000, monthly: 263, label: "Fabric F2" },
      { maxProcessedGB: 30_000, monthly: 526, label: "Fabric F4" },
      { maxProcessedGB: 80_000, monthly: 1_051, label: "Fabric F8" },
      { maxProcessedGB: 200_000, monthly: 2_103, label: "Fabric F16" },
      { maxProcessedGB: 500_000, monthly: 4_206, label: "Fabric F32" },
      { maxProcessedGB: 1_500_000, monthly: 5_257, label: "Fabric F64" },
      {
        maxProcessedGB: Number.POSITIVE_INFINITY,
        monthly: 10_514,
        label: "Fabric F128",
      },
    ],
    recommendedStackUrl:
      "https://learn.microsoft.com/en-us/azure/architecture/example-scenario/data/greenfield-lakehouse-fabric",
  },
  google: {
    key: "google",
    label: "Google Cloud",
    sublabel: "BigQuery + Dataplex + Looker",
    storageRate: 0.02,
    processingRate: 0.0061,
    seatRate: 0,
    // Looker platform fee (~$4,000) is constant; Composer DCU consumption
    // scales with orchestration load.
    platformTiers: [
      {
        maxProcessedGB: 50_000,
        monthly: 4_400,
        label: "Cloud Composer (Small) + Looker",
      },
      {
        maxProcessedGB: 300_000,
        monthly: 4_700,
        label: "Cloud Composer (Medium) + Looker",
      },
      {
        maxProcessedGB: 1_000_000,
        monthly: 5_200,
        label: "Cloud Composer (Large) + Looker",
      },
      {
        maxProcessedGB: Number.POSITIVE_INFINITY,
        monthly: 6_500,
        label: "Cloud Composer (X-Large) + Looker",
      },
    ],
    recommendedStackUrl:
      "https://web.archive.org/web/20260102141530/https://docs.cloud.google.com/architecture/big-data-analytics/analytics-lakehouse",
  },
  amazon: {
    key: "amazon",
    label: "Amazon",
    sublabel: "S3 + Glue + Athena/Redshift + QuickSight",
    storageRate: 0.023,
    processingRate: 0.005,
    // Blended: ~10% QuickSight Authors ($24/mo) + ~90% Readers ($5/mo cap).
    seatRate: 8.0,
    // QuickSight + Glue catalog at small scale; Redshift provisioned at scale.
    platformTiers: [
      {
        maxProcessedGB: 100_000,
        monthly: 250,
        label: "QuickSight + Glue catalog",
      },
      {
        maxProcessedGB: 500_000,
        monthly: 750,
        label: "+ Glue Data Catalog (larger) + Redshift dev",
      },
      {
        maxProcessedGB: 2_000_000,
        monthly: 2_500,
        label: "+ Redshift ra3.xlplus (2-node)",
      },
      {
        maxProcessedGB: Number.POSITIVE_INFINITY,
        monthly: 6_000,
        label: "+ Redshift production cluster",
      },
    ],
    recommendedStackUrl:
      "https://aws.amazon.com/blogs/architecture/how-to-accelerate-building-a-lake-house-architecture-with-aws-glue/",
  },
  firestream: {
    key: "firestream",
    label: "Firestream",
    sublabel: "Apache Airflow + Superset (OSS, on K3s)",
    storageRate: 0.02,
    processingRate: 0.0061,
    seatRate: 0,
    // Honest scaling: a single small VM runs out of Airflow worker capacity
    // around 100 TB/mo of orchestrated pipelines. Beyond that you need
    // more compute — but it scales linearly on cheap hardware, not on
    // licensed capacity tiers.
    platformTiers: [
      { maxProcessedGB: 100_000, monthly: 150, label: "1 small VM" },
      { maxProcessedGB: 500_000, monthly: 300, label: "1 medium VM" },
      { maxProcessedGB: 2_000_000, monthly: 600, label: "2 medium VMs" },
      {
        maxProcessedGB: Number.POSITIVE_INFINITY,
        monthly: 1_500,
        label: "k3s on commodity VMs",
      },
    ],
    recommendedStackUrl: "https://github.com/datawizz/Firestream",
  },
};

// "Force F64" toggle: ignores auto-tier and pins Microsoft to F64 capacity,
// which makes Power BI viewers free (seatRate=0). Useful narrative for
// showing the per-seat / enterprise-capacity tradeoff at any scale.
export const F64_FORCED_TIER: PlatformTier = {
  maxProcessedGB: Number.POSITIVE_INFINITY,
  monthly: 5_257,
  label: "Fabric F64 (forced)",
};

export const STACK_ORDER: StackKey[] = [
  "microsoft",
  "google",
  "amazon",
  "firestream",
];

export const DEFAULT_INPUTS = {
  storedGB: 100,
  processedGB: 1_000,
  seats: 30,
} as const;

export const SLIDER_RANGES = {
  stored: { min: 100, max: 1_000_000 },
  processed: { min: 100, max: 5_000_000 },
  seats: { min: 1, max: 2_000 },
} as const;

export const RATES_AS_OF = "mid-2026";

// Per-stack visual tokens. Vendor color now lives in two surfaces only: a small
// accent dot next to the label, and the chart bar fill. Everything else uses
// the neutral system + emerald-accent design language.
export interface BrandTokens {
  accentBgClass: string; // small dot beside the vendor name
  chartColor: string; // raw CSS color for recharts bars
}

export const BRAND: Record<StackKey, BrandTokens> = {
  microsoft: { accentBgClass: "bg-sky-500", chartColor: "rgb(14 165 233)" },
  google: { accentBgClass: "bg-amber-500", chartColor: "rgb(245 158 11)" },
  amazon: { accentBgClass: "bg-orange-500", chartColor: "rgb(249 115 22)" },
  firestream: {
    accentBgClass: "bg-emerald-500",
    chartColor: "rgb(16 185 129)",
  },
};
