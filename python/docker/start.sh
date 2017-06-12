docker run -it --rm -e DISPLAY -v $HOME/.Xauthority:/home/developer/.Xauthority -v `pwd`:/data --net=host ppandas /bin/bash
