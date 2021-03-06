{
    appDir: "../www",
    baseUrl: "js",
    dir: "../www-release",
    shim : {
        "bootstrap" : { deps :['jquery'] },
        'jquery.imgareaselect': { deps: ['jquery'] }
    },
    paths: {
        jquery: "lib/jquery",
        'jquery.ui.widget': 'lib/jquery.ui.widget',
        'jquery.imgareaselect': 'lib/jquery.imgareaselect',
        bootstrap: "lib/bootstrap",
        underscore: "lib/underscore",
        template: "../template",
        'handlebars.runtime': 'lib/handlebars-runtime'
    },
    modules: [
        {
            name: "../main"
        }
    ],
    optimize: "uglify2",
    preserveLicenseComments: false,
}
