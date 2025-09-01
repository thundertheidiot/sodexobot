.PHONY: docker push deploy

docker:
	docker load < "$$(nix build .#docker --print-out-paths)"

push: docker
	docker push ghcr.io/thundertheidiot/sodexobot

deploy: push
	ssh kotiboksi.xyz 'cd /srv/docker-setup/web-stack && \
	docker-compose pull sodexobot && \
	docker-compose stop sodexobot && \
	docker-compose rm -f sodexobot && \
	docker-compose up -d sodexobot'
