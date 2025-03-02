set -e

# tags
INDEXER_TAG=$(grep '^version' indexer/Cargo.toml | awk -F' = ' '{print $2}' | tr -d '"')
GRAPHQL_TAG=$(grep '^version' graphql/Cargo.toml | awk -F' = ' '{print $2}' | tr -d '"')

# User must be logged in to vultr docker registry
REGISTRY="fra.vultrcr.com/tombola"

# build and push images
ECR_INDEXER_REPOSITORY="indexer"
docker build --platform="linux/arm64" -t $ECR_INDEXER_REPOSITORY:latest -f docker/$ECR_INDEXER_REPOSITORY/Dockerfile .
docker tag $ECR_INDEXER_REPOSITORY:latest $REGISTRY/$ECR_INDEXER_REPOSITORY:latest
docker push $REGISTRY/$ECR_INDEXER_REPOSITORY:latest

ECR_GRAPHQL_REPOSITORY="graphql"
docker build --platform="linux/arm64" -t $ECR_GRAPHQL_REPOSITORY:latest -f docker/$ECR_GRAPHQL_REPOSITORY/Dockerfile .
docker tag $ECR_GRAPHQL_REPOSITORY:latest $REGISTRY/$ECR_GRAPHQL_REPOSITORY:latest
docker push $REGISTRY/$ECR_GRAPHQL_REPOSITORY:latest