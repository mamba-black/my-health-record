#!/usr/bin/env nu

# Generar IDs únicos
let trace_id = (od -An -tx1 -N16 /dev/urandom | tr -d ' \n' | str trim)
let parent_span_id = (od -An -tx1 -N8 /dev/urandom | tr -d ' \n' | str trim)
let child_span_id = (od -An -tx1 -N8 /dev/urandom | tr -d ' \n' | str trim)

# Tiempos
let now_ns = (date now | into int)
let start_ns = ($now_ns - 800_000_000)
let end_ns = $now_ns
let child_start_ns = ($now_ns - 600_000_000)
let child_end_ns = ($now_ns - 200_000_000)

print $"Generando telemetría simulada..."
print $"Trace ID: ($trace_id)"

# 1. Enviar Traza a Tempo (HTTP OTLP)
let trace_payload = {
  resourceSpans: [
    {
      resource: {
        attributes: [
          {
            key: "service.name"
            value: { stringValue: "frontend-service" }
          }
        ]
      }
      scopeSpans: [
        {
          spans: [
            {
              traceId: $trace_id
              spanId: $parent_span_id
              name: "GET /checkout"
              kind: 3 # CLIENT
              startTimeUnixNano: ($start_ns | into string)
              endTimeUnixNano: ($end_ns | into string)
              status: { code: 1 }
            }
          ]
        }
      ]
    }
    {
      resource: {
        attributes: [
          {
            key: "service.name"
            value: { stringValue: "backend-service" }
          }
        ]
      }
      scopeSpans: [
        {
          spans: [
            {
              traceId: $trace_id
              spanId: $child_span_id
              parentSpanId: $parent_span_id
              name: "POST /charge_card"
              kind: 2 # SERVER
              startTimeUnixNano: ($child_start_ns | into string)
              endTimeUnixNano: ($child_end_ns | into string)
              status: { code: 1 }
            }
          ]
        }
      ]
    }
  ]
}

print "Enviando traza a Tempo..."
let trace_res = (curl -s -X POST http://localhost:4318/v1/traces
  -H "Content-Type: application/json"
  -d ($trace_payload | to json))

# 2. Enviar Logs a Loki
let log_payload = {
  streams: [
    {
      stream: {
        service_name: "frontend-service"
        job: "checkout-flow"
      }
      values: [
        [ ($now_ns | into string), $"Iniciando checkout para usuario, trace_id=($trace_id)" ]
      ]
    }
    {
      stream: {
        service_name: "backend-service"
        job: "checkout-flow"
      }
      values: [
        [ (($now_ns + 1_000_000) | into string), $"Procesando pago con tarjeta, trace_id=($trace_id)" ]
      ]
    }
  ]
}

print "Enviando logs a Loki..."
let log_res = (curl -s -X POST http://localhost:3100/loki/api/v1/push
  -H "Content-Type: application/json"
  -d ($log_payload | to json))

# 3. Enviar Métricas de Recursos a Prometheus (HTTP OTLP)
# Simulamos 68% de CPU y 7.5GB de memoria
let metrics_payload = {
  resourceMetrics: [
    {
      resource: {
        attributes: [
          {
            key: "service.name"
            value: { stringValue: "checkout-simulator" }
          }
        ]
      }
      scopeMetrics: [
        {
          metrics: [
            {
              name: "system.cpu.utilization"
              description: "CPU utilization metric"
              unit: "1"
              gauge: {
                dataPoints: [
                  {
                    timeUnixNano: $now_ns
                    asDouble: 0.68
                  }
                ]
              }
            }
            {
              name: "system.memory.usage"
              description: "Memory usage metric"
              unit: "By"
              gauge: {
                dataPoints: [
                  {
                    timeUnixNano: $now_ns
                    asInt: "7516192768"
                  }
                ]
              }
            }
          ]
        }
      ]
    }
  ]
}

print "Enviando métricas de CPU y Memoria a Prometheus..."
let metrics_res = (curl -s -X POST http://localhost:9090/api/v1/otlp/v1/metrics
  -H "Content-Type: application/json"
  -d ($metrics_payload | to json))

print "\n¡Todo enviado con éxito!"
print $"\nVerifica en Grafana:"
print $"- Loki [Logs]: {{service_name=\"backend-service\"}} o busca la frase 'trace_id=($trace_id)'"
print $"- Tempo [Trazas]: Buscar Trace ID '($trace_id)'"
print $"- Prometheus [Métricas]: Buscar 'system_cpu_utilization_ratio' o 'system_memory_usage_bytes'"


