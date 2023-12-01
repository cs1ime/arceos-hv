NIMBOS=/home/huaji/Documents/hv/nimbos-guest/kernel
PREV=`pwd`
cd $NIMBOS
make ARCH=x86_64 LOG=warn
cp target/x86_64/release/nimbos.bin $PREV/apps/hv/guest/nimbos/
cd $PREV

make ARCH=x86_64 A=apps/hv HV=y GUEST=nimbos run
