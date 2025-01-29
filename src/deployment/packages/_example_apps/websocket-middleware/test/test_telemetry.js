

const { credentials } = require('@grpc/grpc-js');
const { NodeSDK } = require('@opentelemetry/sdk-node');
const { OTLPTraceExporter } = require('@opentelemetry/exporter-trace-otlp-proto');

const sdk = new NodeSDK({
    traceExporter: new OTLPTraceExporter({
        url: 'signoz-otel-collector.default.svc.cluster.local:4318',
        credentials: credentials.createInsecure(),
    }),
});

sdk.start();

const { context, trace, SpanKind, diag } = require('@opentelemetry/api');

const { SpanStatusCode } = require('@opentelemetry/api');
const tracer = trace.getTracer('example-tracer');

const span = tracer.startSpan('span_with_logs');
span.setStatus({ code: SpanStatusCode.OK });
span.addEvent('Info log event', { info: 'extra info' });
span.addEvent('Warning log event', { warning: 'extra warning' });
span.end();



// // Metrics
// const { MeterProvider } = require('@opentelemetry/sdk-metrics');
// const meter = new MeterProvider().getMeter('example-meter');

// const counter = meter.createCounter('example_counter', {
//     description: 'Example counter metric',
// });
// const labels = { key: 'value' };
// const boundCounter = counter.bind(labels);
// boundCounter.add(1);

