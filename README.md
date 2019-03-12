# qtree-demo
This is a simple quad-tree-based 2D collision detection implementation. You can
try it out by running:
```shell
$ cargo run --release
```

## Controls and behavior
* `1` - toggle drawing circles
* `2` - toggle drawing circle bounding boxes
* `3` - toggle drawing quad tree subregions
* `left-click` - create a new circle originating at cursor position
* `right-click` - purge all circles
* `middle-click` - Add a bunch of circles for scale testing
* `scroll` - zoom the smallest circle the cursor collides with

Blue color of a circle means it collides with the cursor.
