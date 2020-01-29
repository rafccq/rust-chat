# Simple Rust 1-1 chat

A simple one-on-one chat application. It can run in either a server or client mode. As a server, it will wait for clients to connect. Upon the client's connection termination, it will continue listening for the next client.
Once a connection is stabilized, client and server can send messages to each other. The message receiver will acknowledge it with an ack, and the sender will measure the roundtrip time.

## Usage
``` 
    chat [OPTIONS]

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -h, --host <addr>      IP address to listen for connections (server) or connect to (client)
    -m, --mode <MODE>      server/client
    -p, --port <number>    Port to listen for connections (server) or connect to (client)
```

To end the application, type the command `\quit`

## Protocol

The protocol is very simple: a header indicates the message type (MESSAGE, ACK, DISCONNECTED)
| h | contents |

The byte buffer _contents _ holds the encoded message. When the message type is ACK or DISCONNECTED, contents is empty. Example:

ACK:

| 0xA |

DISCONNECTED:

| 0xC |

MESSAGE "Hello":

| 0xB | 'h' 'e' 'l' 'l' 'o' |

The message contents are converted to a utf8 string on the application level.

## Implementation
When a connection is stablished, both modes will call the `process` function, the client will call it once and return when it's finished, while the server will loop, always waiting for new connections.




## Possible Improvements
- Handle errors in try_send()
- Handle unix signals like SIGINT, that force the termination of the application
- Handle commands (such as "\quit") outside the process_connections() function. Actually, this improvement involves a more general issue: currently, client and server are implemented in the same function, process_connections(), exactly the same way. However, they will probably have different requirements in the future.

