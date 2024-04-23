# wgdhcp

log
```
/wg/client # wg-quick up wg0
Warning: `/etc/wireguard/wg0.conf' is world accessible
[#] ip link add wg0 type wireguard
[#] wg setconf wg0 /dev/fd/63
[#] ip -4 address add 192.168.103.1/16 dev wg0
[#] ip link set mtu 1420 up dev wg0
[#] iptables -A FORWARD -i wg0 -j ACCEPT; iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE;iptables -A FORWARD -o wg0 -j ACCEPT
/wg/client # wg-quick down wg0
Warning: `/etc/wireguard/wg0.conf' is world accessible
[#] ip link delete dev wg0
[#] iptables -D FORWARD -i wg0 -j ACCEPT; iptables -t nat -D POSTROUTING -o eth0 -j MASQUERADE;iptables -D FORWARD -o wg0 -j ACCEPT
```

file
```conf
[Interface]
Address = 192.168.103.1/16
ListenPort = 45340
PrivateKey = <server private key value> # the key from the previously generated privatekey file
PostUp = iptables -A FORWARD -i %i -j ACCEPT; iptables -t nat -A POSTROUTING -o eth0 -j MASQUERADE;iptables -A FORWARD -o %i -j ACCEPT
PostDown = iptables -D FORWARD -i %i -j ACCEPT; iptables -t nat -D POSTROUTING -o eth0 -j MASQUERADE;iptables -D FORWARD -o %i -j ACCEPT

[Peer]
PublicKey = <client public key value> # obtained from client device via wireguard connection setup process
AllowedIPs = 192.168.103.2/16
```