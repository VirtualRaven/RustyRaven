kubectl port-forward -n sjf svc/minio  9000:80  --address=0.0.0.0 &
kubectl port-forward -n sjf svc/cluster-sjf-rw  5433:5432  --address=0.0.0.0 &