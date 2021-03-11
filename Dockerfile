FROM paritytech/ci-linux:974ba3ac-20201006

COPY ./tmp/parallel /usr/local/bin/

EXPOSE 9944

CMD ["parallel", "--dev", "--ws-external"]
