# ssh-agent-notify

A `ssh-agent` proxy that shows notifications when your ssh private keys are used.

This program creates a unix domain socket and bypasses all connections to the actual ssh-agent process.
When clients send [`SSH_AGENTC_SIGN_REQUEST`](https://tools.ietf.org/id/draft-miller-ssh-agent-01.html#rfc.section.4.5),
this program shows notifications via [`libnotify`](https://developer.gnome.org/libnotify/).

## Usage

```
$ echo $SSH_AUTH_SOCK  # make sure ssh-agent is running and $SSH_AUTH_SOCK is set.
$ cargo run --release -- ssh-agent-notify.sock &
$ SSH_AUTH_SOCK=ssh-agent-notify.sock ssh some_host  # connect a host that uses public key authentication.
```
