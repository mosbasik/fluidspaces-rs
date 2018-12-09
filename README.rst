fluidspaces
===========

Usage
-----

This application has two parts; the ``fluidspaces`` daemon and the
``fluidspaces-msg`` binary used to send messages to the daemon.  My setup is to
run the daemon as a systemd user service and bind the various messages to keys.
I use ``sxhkd`` for keybinds, but you can do this just as easily in the ``i3``
config file.

These are the flags expected by ``fluidspaces-msg``:

::

    USAGE:
        fluidspaces-msg [OPTIONS]

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    OPTIONS:
        -a, --action <action>    Action to perform [default: go_to]  [values: go_to, send_to, bring_to, toggle]

And these are some examples of valid invocations of ``fluidspaces-msg``:

.. code-block:: bash

    # pick a workspace to go to
    fluidspaces-msg --action go_to

    # pick a workspace to send the currently active container to
    fluidspaces-msg --action send_to

    # pick a workspace to go to and bring the currently active container with you
    fluidspaces-msg -a bring_to

    # go to the most recent non-active workspace
    fluidspaces-msg -a toggle

Installation
------------

Arch
~~~~

Install the ``fluidspaces-rs-git`` package from the `AUR <https://aur.archlinux.org/>`_

Add the following to your ``~/.xinitrc``:

.. code-block:: bash

    # make DISPLAY available to systemd user services
    systemctl --user import-environment DISPLAY

    # start fluidspaces daemon
    systemctl --user start fluidspaces.service

From Source
~~~~~~~~~~~

*Todo*
