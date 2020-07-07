FROM teaclave/teaclave-build-ubuntu-1804-sgx-2.9.1

WORKDIR /teaclave
COPY . /teaclave

# make directory to store kds
RUN mkdir /var/xuperdata

# set aesm_service path
ENV LD_LIBRARY_PATH /opt/intel/sgx-aesm-service/aesm

# start aesm_service and fns
CMD /teaclave/start.sh
