# Basic UDP server/client

## Summary
This utility is a simple combo server/client to transfer files reliably over UDP.  It serves files listed on a whitelist that can be specified via a config file.  It's simple, secure and very easy to use!


## Usage
### Server
basic_udp &lt;config file name&gt;

### Client (Currently just grabs a file and saves it as Testout)
basic_udp &lt;IP:port&gt; &lt;filename&gt; &lt;outfilename&gt;


## Config file
The config file is a series of key value pairs specified on subsequent lines

### For example
ip 127.0.0.1:9001
whitelist whitelist
### Would specify a config equivalent to the default settings
### If a config file is missing one of the required settings, the server will use defaults and print out a message about it



## Design goals
This will be a stateless microservice friendly file transfer utility that runs over UDP.  It's lightweight, clients request ranges of chunks in a file and servers send back UDP packets that are mostly file data.


## Next tasks
This project is very new to me, and in a very early state.  A lot of development will be proof of concept but I will start branching and avoid pushing breaking changes.  Next up
- Create a container to make deployment easy

And more, we'll see where this goes.  Thanks for reading!


## Contributing
I love the enthusiasm, I work on this project as a hobby alongside my day job.  I don't have a lot of time to look into pull requests or develop this in any serious capacity yet.  If it picks up steam over time, I might look into treating this like a proper open source project and developing some sort of policy here but for now it's just best effort.
