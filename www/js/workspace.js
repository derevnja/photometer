define( function(require) {

    var Backbone = require( "lib/backbone" );

    var Workspace = Backbone.Router.extend({
        routes: {
            'login': 'login',
            'register': 'register'
        },

        login: function() {
            var UserLoginView = require( "login/view" ),
                UserLoginModel = require( "login/model" );

            this.current = new UserLoginView( { model: new UserLoginModel } );
        },

        register: function() {
            var RegisterView = require( "register/view" ),
                RegisterModel = require( "register/model" );

            this.current = new RegisterView( { model: new RegisterModel } );
        },

    });

    return Workspace;

});