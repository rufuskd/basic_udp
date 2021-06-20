# Basic UDP server/client

## Summary
This utility is a simple combo server/client to transfer files reliably over UDP.  In it's current state, I wouldn't advise running it facing the internet, it serves up any file you ask for without any limits at all!  In the future it'll run using a whitelist of files to serve.  It's intended to be a stateless file server that can be run in containers as a microservice.



## Usage
### Server
basic_udp &ltIP:port&gt

### Client (Currently just grabs a file and saves it as Testout)
basic_udp &ltIP:port&gt &ltfilename&gt


## Design goals
This will be a stateless microservice friendly file transfer utility that runs over UDP.  It's lightweight, client's request ranges of chunks in a file and servers send back UDP packets that are mostly file data.


## Next tasks
This project is very new to me, and in a very early state.  A lot of development will be proof of concept but I will start branching and avoid pushing breaking changes.  Next up
- Add an argument to specify outfile
- Set up a whitelit system to serve files more securely
- Set up a config file system
- Create a container to make deployment easy

And more, we'll see where this goes.  Thanks for reading!

## Contributing
I love the enthusiasm, I work on this project as a hobby alongside my day job.  I don't have a lot of time to look into pull requests or develop this in any serious capacity yet.  If it picks up steam over time, I might look into treating this like a proper open source project and developing some sort of policy here but for now it's just best effort.