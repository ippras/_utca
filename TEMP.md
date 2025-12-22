если вы настраиваете ключи для пользователя-администратора, добавление открытого
ключа в `%USERPROFILE%/.ssh/authorized_keys` не будет работать. Вместо этого
необходимо добавить открытый ключ в другой файл
`%PROGRAMDATA%/ssh/administrators_authorized_keys`.

```
$acl = Get-Acl C:\ProgramData\ssh\administrators_authorized_keys
$acl.SetAccessRuleProtection($true, $false)
$administratorsRule = New-Object system.security.accesscontrol.filesystemaccessrule("Administrators","FullControl","Allow")
$systemRule = New-Object system.security.accesscontrol.filesystemaccessrule("SYSTEM","FullControl","Allow")
$acl.SetAccessRule($administratorsRule)
$acl.SetAccessRule($systemRule)
$acl | Set-Acl
```

Set-service ssh-agent StartupType 'Automatic'
Set-Service ssh-agent -StartupType Automatic

Start-Service ssh-agent

ssh-add "C:/Users/g/.ssh/id_ed25519"

`get-acl "$env:programdata\ssh\ssh_host_rsa_key" | set-acl "$env:programdata\ssh\administrators_authorized_keys"`

`restart-service sshd`

`ssh g@tcp://qnrmq-89-221-48-50.a.free.pinggy.link:42197 -i "C:\Users\g\.ssh\id_ed25519"`

ssh-keygen -p -f "C:\Users\g\.ssh\id_ed25519" -m pem