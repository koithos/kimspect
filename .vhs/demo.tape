Require kubectl
Set Shell zsh
Set FontSize 13
Set FontFamily "Mononoki"
Set Padding 10
Set LineHeight 1.2
Set Width 1920
Set Height 600
Set TypingSpeed 120ms
Set WindowBar Colorful
Set BorderRadius 10
Set WaitTimeout 30s
Output demo.gif
Sleep 3s

Type "kubectl get pods -n argo"
Enter
Wait
Sleep 1s
Type "kimspect get images -n argo"
Enter
Wait
Sleep 1s
Type "kimspect get images -n argo -o wide"
Enter
Wait
Sleep 1s
Type "kimspect get images --registry docker.io --namespace kube-system"
Enter
Wait
Sleep 5s
