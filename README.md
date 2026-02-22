# nur - a taskrunner based on `nu` shell

`nur` is a simple, yet very powerful task runner. It borrows ideas from [`b5`](https://github.com/team23/b5)
and [`just`](https://github.com/casey/just), but uses [`nu` shell scripting](https://www.nushell.sh/book/programming_in_nu.md)
to define the tasks. This allows for well-structured tasks while being able to use the super-powers of `nu`
in your tasks.

## Quick overview and example

![Demo session](assets/demo.gif)

`nur` allows you to execute tasks defined in a file called `nurfile`. It will look through your
current working directory and all its parents to look for this file. When it has found the `nurfile`
it will change to the directory the file was found in and then `source` the file into `nu` script.
You can define tasks like this:

```nu-script
# Just tell anybody or the "world" hello
def "nur hello" [
    name: string = "world"  # The name to say hello to
] {
    print $"hello ($name)"
}
```

The important bit is that you define your tasks as subcommands for "nur". If you then execute
`nur hello` it will print "hello world", meaning it did execute the task `hello` in your `nurfile`.
You can also use `nur --help` to get some details on how to use `nur` and `nur --help hello` to
see what this `hello` task accepts as parameters.

You may also pass arguments to your `nur` tasks, like using `nur hello bob` to pass "bob"
as the name to the "hello" task. This supports all parameter variants normal `nu` scripts could also
handle. You may use `nur --help <task-name>` to see the help for an available command.

Your tasks then can do whatever you want them to do in `nu` script. This allows for very structured
usage of for example docker to run/manage your project needs. But it can also execute simple commands
like you would normally do in your shell (like `npm ci` or something). `nur` is not tied to any
programming language, packaging system or anything. As in the end the `nurfile` is basically a
normal `nu` script you can put into this whatever you like.

I recommend reading [working with `nur`](https://nur-taskrunner.github.io/docs/working-with-nur/) to get an
overview how to use `nur`. Also I recommend reading the `nu` documentation about
[custom commands](https://www.nushell.sh/book/custom_commands.html) for details on how to define `nu`
commands (and `nur` tasks) and at least read through the
[nu quick tour](https://www.nushell.sh/book/quick_tour.html) to understand some basics and benefits
about `nu` scripting.

## Installing `nur`

You may use `cargo` to quickly install `nur` for your current user:

```shell
> cargo install nur
```

The `nur` binary will be added in `$HOME/.cargo/bin` (or `$"($env.HOME)/.cargo/bin"` in `nu` shell).
Make sure to add this to `$PATH` (or `$env.PATH` in `nu` shell).

For more details see [the `nur` installation docs](https://nur-taskrunner.github.io/docs/installation.html).
This also includes MacOS (using homebrew) and Windows (using `.msi` installer) installation methods.

## Working with `nur`

`nur` uses a file called `nurfile` to define your tasks. This file is a normal `nu` script and may
include any `nur` tasks defined as sub commands to `"nur"`. `nur` tasks may use the normal `nu` command
features to define required arguments, their types and more.

See the [working with `nur`](https://nur-taskrunner.github.io/docs/working-with-nur/) documentation
for more details.

## Switching to `nur`

Switching to `nur` on a large project or when having many projects can be some hassle. The recommended workflow
is to create a `nurfile` that only calls the old task runner and then gradually convert your tasks to be rewritten
as `nur` tasks.

To simplify this process you may use the script [`nurify`](scripts/nurify.nu) to generate a `nurfile` from
many existing task runners.

For more details see the [switching to nur](https://nur-taskrunner.github.io/docs/switching-to-nur.html)
documentation.

## Contributing

See the [contributing](CONTRIBUTING.md) documentation for more details.
