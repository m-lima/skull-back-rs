docker build -t skull:rs .
docker tag skull:rs skull:latest
docker stop skull
docker rm skull
docker create \
  --name skull \
  --volume skull-data:/data \
  --net fly \
  skull
