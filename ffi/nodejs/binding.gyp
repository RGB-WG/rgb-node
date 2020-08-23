{
  "targets": [
    {
      "target_name": "rgb_node",
      "sources": [ "swig_wrap.cxx" ],
      "libraries": [ '-L<(module_root_dir)/../../target/debug/', '-lrgb'],
      'include_dirs': [
          '../',
       ],
      "cflags!": ["-std=c++11"],
    }
  ]
}
