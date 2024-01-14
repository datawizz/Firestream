CONTAINER_REGISTRY_URL = $(shell echo $$CONTAINER_REGISTRY_URL)
CONTAINER_REGISTRY_PORT = $(shell echo $$CONTAINER_REGISTRY_PORT)
SERVICE_NAME = $(shell echo "websocket-middleware")
GIT_COMMIT_HASH = $(shell echo $$GIT_COMMIT_HASH)

export IMAGE_TAG = $(CONTAINER_REGISTRY_URL):$(CONTAINER_REGISTRY_PORT)/$(SERVICE_NAME):$(GIT_COMMIT_HASH)



run_follow:
	# Run the websocket-middleware in follow mode
	# TODO There are more elegant ways to run a multi node cluster of fireworks
	export LEAD_FOLLOW=FOLLOW && \
	node --require ./src/instrumentation.js src/server.js

run_lead:
	# Run the websocket-middleware in lead mode
	export LEAD_FOLLOW=LEAD && \
	node --require ./src/instrumentation.js src/server.js

build:
	docker build --no-cache \
		--build-arg SERVICE_NAME=$SERVICE_NAME \
		. \
		-t $(IMAGE_TAG)

push:
	docker push $(IMAGE_TAG)

.PHONY: build run push test clean