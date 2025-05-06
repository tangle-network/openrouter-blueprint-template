# OpenRouter Blueprint Deployment Guide

This guide provides instructions for deploying the OpenRouter Blueprint to various environments. The OpenRouter Blueprint is designed to be deployed as a provider on the Tangle network, enabling it to balance requests across locally hosted LLMs.

## Deployment Options

There are several ways to deploy the OpenRouter Blueprint:

1. **Local Development**: Run the blueprint locally for development and testing
2. **Standalone Server**: Deploy the blueprint as a standalone server
3. **Tangle Network**: Deploy the blueprint as a provider on the Tangle network

## Prerequisites

Before deploying the OpenRouter Blueprint, ensure you have:

- Rust toolchain (1.70+)
- Access to one or more LLM instances
- Tangle node (for Tangle network deployment)
- Configuration file or environment variables set up

## Local Development Deployment

For local development and testing, you can run the blueprint directly:

1. Build the project:

```bash
cargo build
```

2. Create a configuration file:

```bash
cp config.sample.json config.json
```

3. Edit the configuration file to match your local environment.

4. Run the blueprint:

```bash
cargo run -- --config config.json
```

## Standalone Server Deployment

For a standalone server deployment, you'll want to build a release version:

1. Build the project in release mode:

```bash
cargo build --release
```

2. Create a configuration file:

```bash
cp config.sample.json /etc/open-router-blueprint/config.json
```

3. Edit the configuration file to match your server environment.

4. Create a systemd service file at `/etc/systemd/system/open-router-blueprint.service`:

```ini
[Unit]
Description=OpenRouter Blueprint Service
After=network.target

[Service]
Type=simple
User=openrouter
ExecStart=/path/to/open-router-blueprint-template-bin --config /etc/open-router-blueprint/config.json
Restart=on-failure
RestartSec=5s
LimitNOFILE=4096

[Install]
WantedBy=multi-user.target
```

5. Enable and start the service:

```bash
sudo systemctl enable open-router-blueprint
sudo systemctl start open-router-blueprint
```

## Tangle Network Deployment

To deploy the OpenRouter Blueprint as a provider on the Tangle network:

1. Build the project in release mode:

```bash
cargo build --release
```

2. Create a configuration file:

```bash
cp config.sample.json /etc/open-router-blueprint/config.json
```

3. Edit the configuration file to match your environment.

4. Set up a Tangle account and obtain a service ID.

5. Create a systemd service file at `/etc/systemd/system/open-router-blueprint.service`:

```ini
[Unit]
Description=OpenRouter Blueprint Service
After=network.target

[Service]
Type=simple
User=openrouter
Environment="TANGLE_SERVICE_ID=your-service-id"
Environment="TANGLE_NODE_URL=wss://testnet.tangle.tools"
Environment="TANGLE_KEYSTORE_PATH=/etc/open-router-blueprint/keystore"
ExecStart=/path/to/open-router-blueprint-template-bin --config /etc/open-router-blueprint/config.json
Restart=on-failure
RestartSec=5s
LimitNOFILE=4096

[Install]
WantedBy=multi-user.target
```

6. Enable and start the service:

```bash
sudo systemctl enable open-router-blueprint
sudo systemctl start open-router-blueprint
```

## Docker Deployment

You can also deploy the OpenRouter Blueprint using Docker:

1. Create a Dockerfile:

```Dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
WORKDIR /app
COPY --from=builder /app/target/release/open-router-blueprint-template-bin /app/
COPY config.sample.json /app/config.json
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
EXPOSE 3000
CMD ["/app/open-router-blueprint-template-bin", "--config", "/app/config.json"]
```

2. Build the Docker image:

```bash
docker build -t open-router-blueprint .
```

3. Run the Docker container:

```bash
docker run -p 3000:3000 -v /path/to/config.json:/app/config.json open-router-blueprint
```

## Kubernetes Deployment

For a production deployment on Kubernetes:

1. Create a ConfigMap for the configuration:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: open-router-blueprint-config
data:
  config.json: |
    {
      "llm": {
        "api_url": "http://llm-service:8000",
        "timeout_seconds": 60,
        "max_concurrent_requests": 10
      },
      "load_balancer": {
        "strategy": "LeastLoaded",
        "max_retries": 3,
        "selection_timeout_ms": 1000
      },
      "api": {
        "enabled": true,
        "host": "0.0.0.0",
        "port": 3000,
        "auth_enabled": true,
        "rate_limiting_enabled": true,
        "max_requests_per_minute": 100,
        "metrics_interval_seconds": 60
      }
    }
```

2. Create a Secret for the API key:

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: open-router-blueprint-secrets
type: Opaque
stringData:
  api-key: "your-api-key"
  tangle-key: "your-tangle-key"
```

3. Create a Deployment:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: open-router-blueprint
  labels:
    app: open-router-blueprint
spec:
  replicas: 3
  selector:
    matchLabels:
      app: open-router-blueprint
  template:
    metadata:
      labels:
        app: open-router-blueprint
    spec:
      containers:
      - name: open-router-blueprint
        image: open-router-blueprint:latest
        ports:
        - containerPort: 3000
        volumeMounts:
        - name: config-volume
          mountPath: /app/config.json
          subPath: config.json
        env:
        - name: OPENROUTER_API_AUTH_TOKEN
          valueFrom:
            secretKeyRef:
              name: open-router-blueprint-secrets
              key: api-key
        - name: TANGLE_SERVICE_ID
          value: "your-service-id"
        - name: TANGLE_NODE_URL
          value: "wss://testnet.tangle.tools"
        resources:
          limits:
            cpu: "1"
            memory: "1Gi"
          requests:
            cpu: "500m"
            memory: "512Mi"
      volumes:
      - name: config-volume
        configMap:
          name: open-router-blueprint-config
```

4. Create a Service:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: open-router-blueprint
spec:
  selector:
    app: open-router-blueprint
  ports:
  - port: 3000
    targetPort: 3000
  type: ClusterIP
```

5. Create an Ingress (optional):

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: open-router-blueprint
  annotations:
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  rules:
  - host: openrouter.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: open-router-blueprint
            port:
              number: 3000
  tls:
  - hosts:
    - openrouter.example.com
    secretName: openrouter-tls
```

## Monitoring and Logging

For monitoring and logging, you can use:

1. **Prometheus**: The OpenRouter Blueprint exposes metrics that can be scraped by Prometheus.

2. **Grafana**: Create dashboards to visualize the metrics.

3. **ELK Stack**: Use Elasticsearch, Logstash, and Kibana for log aggregation and analysis.

## Security Considerations

When deploying the OpenRouter Blueprint, consider the following security best practices:

1. **Enable Authentication**: Always enable authentication in production environments.

2. **Use HTTPS**: Deploy behind a reverse proxy that terminates HTTPS.

3. **Rate Limiting**: Enable rate limiting to prevent abuse.

4. **Firewall Rules**: Restrict access to the API server to trusted IPs.

5. **Secrets Management**: Use a secrets management solution like HashiCorp Vault or Kubernetes Secrets.

## Scaling Considerations

To scale the OpenRouter Blueprint:

1. **Horizontal Scaling**: Deploy multiple instances behind a load balancer.

2. **Vertical Scaling**: Increase the resources (CPU, memory) allocated to each instance.

3. **Load Balancing**: Use the built-in load balancing capabilities to distribute requests across multiple LLM nodes.

## Troubleshooting

If you encounter issues with your deployment:

1. **Check Logs**: Look for error messages in the logs.

2. **Verify Configuration**: Ensure your configuration file is valid.

3. **Check Connectivity**: Verify that the blueprint can connect to the LLM API.

4. **Monitor Resources**: Check CPU, memory, and network usage.

## Upgrading

To upgrade the OpenRouter Blueprint:

1. Build the new version:

```bash
git pull
cargo build --release
```

2. Stop the service:

```bash
sudo systemctl stop open-router-blueprint
```

3. Replace the binary:

```bash
sudo cp target/release/open-router-blueprint-template-bin /path/to/installation/
```

4. Start the service:

```bash
sudo systemctl start open-router-blueprint
```

For Kubernetes deployments, simply update the image tag in your deployment manifest and apply the changes:

```bash
kubectl apply -f deployment.yaml
```
