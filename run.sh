#!/bin/bash

if [[ $# -gt 0 ]]; then
	BUILD=0
	RUN=0
else
	BUILD=0
	RUN=1
fi

while [[ $# -gt 0 ]]; do
	case $1 in
		-b|--build)
			BUILD=1
			shift
			;;
		-r|--run)
			RUN=1
			shift
			;;
		--default)
			echo "Invalid command line arguments"
			echo
			echo "$0 [OPTIONS]"
			echo "OPTION:"
			echo "-b      - build container"
			echo "-r      - run solution"
			shift # past argument
			;;
	esac
done

# Variables
IMAGE_NAME=orderbook
CONT_NAME=orderbook_cont

RUST_VERSION_1=stable
RUST_VERSION_2=1.31

# Build container
build_container () {
	docker build . -t ${IMAGE_NAME}:latest

	docker ps -a | grep ${CONT_NAME}
	if [ $? -eq 0 ]; then
		docker container stop ${CONT_NAME}
		docker container rm ${CONT_NAME}
	fi
}

# Run container
start_container () {
	docker ps | grep ${CONT_NAME}
	if [ $? -ne 0 ]; then
		docker ps -a | grep ${CONT_NAME}
		if [ $? -ne 0 ]; then
			docker run -dit --name ${CONT_NAME} ${IMAGE_NAME}
		else
			docker start ${CONT_NAME}
		fi
	fi
}

solution () {
	# Solution Build
	docker exec -it ${CONT_NAME} sh -c "cargo build"

	# Solution Unit tests
	docker exec -it orderbook_cont sh -c "cargo test"
}

main () {
	if [ ${BUILD} -eq 1 ]; then
		build_container
		# After building new image remove older containers
	fi

	if [ ${RUN} -eq 1 ]; then
		start_container
		solution
	fi
}

main
