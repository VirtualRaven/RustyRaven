kubectl port-forward -n sjf svc/garage  9000:s3-api  --address=0.0.0.0 &
kubectl port-forward -n sjf svc/cluster-sjf-rw  5433:5432  --address=0.0.0.0 &