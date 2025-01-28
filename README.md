# gluon-image-info

This is a simple tool to extract information from a gziped x86-64 gluon image.

Example:
```
# cargo build
# wget https://firmware.ffh.zone/vH39.pre/sysupgrade/gluon-ffh-vH39.pre-x86-64-sysupgrade.img.gz
# ./target/debug/gluon-image-info gluon-ffh-vH39.pre-x86-64-sysupgrade.img.gz
openwrt-release: 23.05-SNAPSHOT
gluon-version: v2023.2.4
gluon-release: vH39.pre
site-version: vH37-8-g26dac4a
autoupdater-default-branch: wireguard
autoupdater-default-enabled: true
```

## Internals

- The image ungziped into memory.
- The MBR or GPT table is read.
- The second partition is used.
- The squashfs is read and some files are evaluated.
