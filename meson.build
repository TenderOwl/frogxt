project(
  'frog',
  'rust',
  version: '2.0.0',
  meson_version: '>= 1.5.0',
  default_options: [
    'warning_level=2',
    'werror=false',
  ],
)

app_id = 'com.tenderowl.' + meson.project_name()

i18n = import('i18n')
gnome = import('gnome')

pkgdatadir = get_option('prefix') / get_option('datadir') / meson.project_name()

version = meson.project_version()

if get_option('profile') == 'development'
  profile = 'Devel'
  vcs_tag = run_command('git', 'rev-parse', '--short', 'HEAD').stdout().strip()
  if vcs_tag == ''
    version_suffix = '-devel'
  else
    version_suffix = '-@0@'.format(vcs_tag)
  endif
  application_id = '@0@.@1@'.format(app_id, profile)
else
  profile = ''
  version_suffix = ''
  application_id = app_id
endif

subdir('data')
subdir('src')
subdir('po')

gnome.post_install(
  glib_compile_schemas: true,
  gtk_update_icon_cache: true,
  update_desktop_database: true,
)
