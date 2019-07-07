# ssh-agent-notify

A `ssh-agent` proxy that shows notifications when your ssh private keys are used.

## Usage

```
$ echo $SSH_AUTH_SOCK  # make sure ssh-agent is running and $SSH_AUTH_SOCK is set.
$ cargo +nightly run --release -- ssh-agent-notify.sock &
$ SSH_AUTH_SOCK=ssh-agent-notify.sock ssh some_host  # connect a host that uses public key authentication.
```
