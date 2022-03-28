module.exports = {
  mode: 'jit',
  purge: {
    enabled: false,
    content: [
      // '/home/miuler/proyectos/Medical/frontend/my-health-record.ui/front/target/scala-3.1.1/my-health-record-ui-fastopt/main.js',
      '/home/miuler/proyectos/Medical/frontend/my-health-record.ui/front/target/scala-3.1.1/scalajs-bundler/main/my-health-record-ui-fastopt-bundle.js',
      './my-health-record-ui-fastopt-bundle.js'
    ]
  },
  // darkMode: false, // or 'media' or 'class'
  theme: {
    extend: {},
  },
  variants: {
    extend: {},
  },
  plugins: [],
}
