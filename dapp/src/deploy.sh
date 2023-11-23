URL="http://18.162.168.78:8114"

rm -rf migrations
mkdir migrations
ckb-cli --url ${URL} deploy gen-txs \
    --deployment-config ./deploy.toml \
    --migration-dir ./migrations \
    --from-address ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqt4z78ng4yutl5u6xsv27ht6q08mhujf8s2r0n40 \
    --sign-now \
    --info-file info.json
ckb-cli --url ${URL} deploy sign-txs \
    --from-account ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqt4z78ng4yutl5u6xsv27ht6q08mhujf8s2r0n40 \
    --add-signatures \
    --info-file info.json
ckb-cli --url ${URL} deploy apply-txs --migration-dir ./migrations --info-file info.json
rm -rf info.json
rm -rf migrations
