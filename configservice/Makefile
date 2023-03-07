REGISTRY ?= acr4yrmmmgrhcdk6.azurecr.io
REPOSITORY ?= aheld
IMAGE_VERSION ?= $(shell awk -F ' = ' '$$1 ~ /version/ { gsub(/[\"]/, "", $$2); printf("%s",$$2) }' Cargo.toml)
IMAGE_MAJOR_VERSION = $(shell echo "$(IMAGE_VERSION)" | cut -d '.' -f1 )
IMAGE_MINOR_VERSION = $(shell echo "$(IMAGE_VERSION)" | cut -d '.' -f2 )
IMAGE = $(REGISTRY)/$(REPOSITORY)/hello-kubernetes


.PHONY: build-images
build-images: build-image-linux

.PHONY: build-image-linux
build-image-linux:
	docker build --no-cache\
		-m 9g \
		--platform=linux/amd64 \
		--build-arg IMAGE_VERSION="$(IMAGE_VERSION)" \
		--build-arg IMAGE_CREATE_DATE="`date -u +"%Y-%m-%dT%H:%M:%SZ"`" \
		--build-arg IMAGE_SOURCE_REVISION="`git rev-parse HEAD`" \
		-f Dockerfile -t "$(IMAGE):$(IMAGE_VERSION)" .;

.PHONY: push-image
push-image:
	docker tag $(IMAGE):$(IMAGE_VERSION) $(IMAGE):$(IMAGE_MAJOR_VERSION); \
	docker tag $(IMAGE):$(IMAGE_VERSION) $(IMAGE):$(IMAGE_MAJOR_VERSION).$(IMAGE_MINOR_VERSION); \
	docker push $(IMAGE):$(IMAGE_VERSION); \
	docker push $(IMAGE):$(IMAGE_MAJOR_VERSION); \
	docker push $(IMAGE):$(IMAGE_MAJOR_VERSION).$(IMAGE_MINOR_VERSION)

run-image:
	docker run -ti "$(IMAGE):$(IMAGE_VERSION)"