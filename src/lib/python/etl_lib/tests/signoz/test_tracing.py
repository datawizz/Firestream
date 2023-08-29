from opentelemetry import trace
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
import time


def test_tracing():


    trace.set_tracer_provider(TracerProvider())
    tracer = trace.get_tracer(__name__)

    _HOST = "signoz-otel-collector.default.svc.cluster.local:4317"

    # Configure the OTLP exporter and BatchSpanProcessor to send data to the OpenTelemetry collector
    otlp_exporter = OTLPSpanExporter(endpoint=_HOST, insecure=True) # TODO SSL on everything
    span_processor = BatchSpanProcessor(otlp_exporter)
    trace.get_tracer_provider().add_span_processor(span_processor)

    # Define a function to simulate waiting for a network request
    def network_request():
        with tracer.start_as_current_span("network_request"):
            time.sleep(1) # Simulate waiting for network request

    # Trace the function call
    with tracer.start_as_current_span("main"):
        network_request()

if __name__ == "__main__":
    import pytest
    pytest.main([__file__])