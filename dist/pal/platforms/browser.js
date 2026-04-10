/**
 * Desktop browser (testing)
 */
(function (global) {
    'use strict';

    var P = global.CosmicClimberPAL.PLATFORM_IDS;

    global.CosmicClimberPAL.registerPlatform(P.BROWSER, {
        keyMapping: {
            up: [38, 87],
            down: [40, 83],
            left: [37, 65],
            right: [39, 68],
            action: [13, 32],
            back: [4, 27, 8, 11]
        }
    });
})(typeof window !== 'undefined' ? window : this);
