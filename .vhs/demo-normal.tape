Require kubectl
Set Shell zsh
Set FontSize 16
Set FontFamily "Mononoki"
Set Padding 10
Set LineHeight 1.2
Set Width 1280
Set Height 700
Set TypingSpeed 120ms
Set WindowBar Colorful
Set BorderRadius 10
Set WaitTimeout 30s
Output demo-normal.gif
Sleep 3s

Type "kubectl get pods -n argo"
Enter
Wait
Sleep 1s
Type "kimspect get images -n argo"
Enter
Wait
Sleep 1s
Type "kimspect get images --registry docker.io -n argo"
Enter
Wait
Sleep 5s
