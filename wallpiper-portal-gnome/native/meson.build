project('wallpiper-gnome', 'c',
  default_options: ['warning_level=2', 'c_std=gnu11', 'prefix=/usr'],
)

gnome = import('gnome')

mutter_dep = dependency('libmutter-18')
cogl_dep = dependency('mutter-cogl-18')
clutter_dep = dependency('mutter-clutter-18')
gobject_dep = dependency('gobject-2.0')
gio_unix_dep = dependency('gio-unix-2.0')
gbm_dep = dependency('gbm')
egl_dep = dependency('egl')
drm_dep = dependency('libdrm')

inc = include_directories('include')

lib_sources = [
  'src/wallpiper_portal.c',
  'src/wallpiper_test_actor.c',
  'src/actor_stacking.c',
  'src/capture_listener.c',
  'src/ctl_listener.c',
  'src/egl_import.c',
  'src/error.c',
  'src/monitor_geometry.c',
]

lib = shared_library('wallpiper-1.0',
  lib_sources,
  include_directories: inc,
  dependencies: [mutter_dep, cogl_dep, clutter_dep, gobject_dep, gio_unix_dep, gbm_dep, egl_dep, drm_dep],
  install: true,
)

# Only the files containing the public, GI-exposed API need to be scanned:
# g-ir-scanner compiles these with the same flags/deps as `lib` and follows
# their #includes to resolve types, but only extracts doc comments and
# declarations from the files listed here.
gnome.generate_gir(lib,
  sources: ['include/wallpiper_portal.h', 'src/wallpiper_portal.c', 'src/wallpiper_test_actor.c'],
  namespace: 'Wallpiper',
  nsversion: '1.0',
  identifier_prefix: 'Wallpiper',
  symbol_prefix: 'wallpiper',
  includes: ['GObject-2.0'],
  install: true,
)

executable('wallpiper-test-producer',
  'tools/test_producer.c',
  dependencies: [gbm_dep, drm_dep],
  install: false,
)
