# serial

Run `dmesg` and look for output like below:

```
[ 5804.842412] usb 1-2: new full-speed USB device number 94 using xhci_hcd
[ 5804.992194] usb 1-2: New USB device found, idVendor=5824, idProduct=27dd, bcdDevice= 0.10
[ 5804.992197] usb 1-2: New USB device strings: Mfr=1, Product=2, SerialNumber=3
[ 5804.992199] usb 1-2: Product: Serial port
[ 5804.992200] usb 1-2: Manufacturer: Fake company
[ 5804.992202] usb 1-2: SerialNumber: TEST
[ 5805.036702] cdc_acm 1-2:1.0: ttyACM0: USB ACM device
[ 5805.037540] usbcore: registered new interface driver cdc_acm
[ 5805.037541] cdc_acm: USB Abstract Control Model driver for USB modems and ISDN adapters
```

Take the port found and connect to it with a serial console. I recommend
`picocom`.

> Note:
> You will likely need to use `sudo` to run this command.

```bash
picocom /dev/ttyACM0
```

Once connected, anything typed should be echoed in uppercase.
