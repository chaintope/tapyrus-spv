#! /bin/bash

PROJECT_PATH=$(echo $PWD | sed -e 's/[^a-zA-Z4-9_.-]//g')
CONTAINER="tapyrus-spv-ndk-build-on-${PROJECT_PATH}"
IMAGE=rantan39/android-ndk-and-rustc:r20-latest

if docker ps -f name=${CONTAINER} | grep -w ${CONTAINER} >/dev/null ; then
  echo "Build is already running."
  exit 0
fi

if docker ps -a -f name=${CONTAINER} | grep -w ${CONTAINER} >/dev/null ; then
  echo "Container ${CONTAINER} is exist. start this container."
  docker start -ai ${CONTAINER}
else
  echo "Container ${CONTAINER} is not exist. pull and run ${IMAGE}"
  docker pull ${IMAGE}
  docker run -it -v $PWD:/project --name ${CONTAINER} ${IMAGE}
fi