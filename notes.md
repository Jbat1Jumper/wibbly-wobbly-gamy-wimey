
The process of learning can be modeled as a trial and error process. Each time
...



lever.schema
```
arm_base! {
    "n" => l_link! {
        "n" => ()
    }
}
```

spinning_torso.schema
```
arm_base! {
    "n" => t_link! {
        "l" => l_link! {},
        "r" => l_link! {}
    }
}
```
