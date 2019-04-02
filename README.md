
# GitLocalStats

--------------------

# Dependencies

```sh
~$ go get -u gopkg.in/src-d/go-git.v4/... # For git api
~$ go get -u github.com/joho/godotenv     # For configuration
```

--------------------

# Usage

If you want to use this project, mhake sure there is a .gitlocalstats file in your home directory that contains
your email and the path to your repos.

```
email=my-email
folder=/path/to/repos
```

The project will recursively scan through the file system looking for git repos starting with the
folder you specify in .gitlocalstats. 

```
~$ gitlocalstats
```

or just alias the command because it's long

```
~$ alias gls='gitlocalstats'
~$ gls
```


