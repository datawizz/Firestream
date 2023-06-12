# Downloads example datasets for testing the project
#TODO consider storing these in the project's artifact store, for posterity.

mkdir -p /workspace/opt/data
cd /workspace/opt/data
curl "https://dl.fbaipublicfiles.com/fasttext/vectors-crawl/cc.en.300.vec.gz" | gunzip | tail -n +2 | gzip > data.gz 



http://download.bls.gov/pub/time.series/wp/

https://download.bls.gov/pub/time.series/pc/