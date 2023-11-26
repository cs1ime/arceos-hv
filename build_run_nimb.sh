NIMBOS=/home/huaji/Documents/myarceos/nimbos-guest/kernel
PREV=`pwd`
cd $NIMBOS
make ARCH=x86_64 LOG=warn
cp target/x86_64/release/nimbos.bin $PREV/apps/hv/guest/nimbos/
cd $PREV

make ARCH=x86_64 A=apps/hv HV=y LOG=info GUEST=nimbos run
