/**
 * Shared namespace for modular TV PAL (Cosmic Climber).
 * Load first, before detect-platform.js and platform modules.
 */
(function (global) {
    'use strict';

    var PLATFORM_IDS = {
        TIZEN: 'tizen',
        WEBOS: 'webos',
        VIZIO: 'vizio',
        FIRETV: 'firetv',
        ANDROID_TV: 'android_tv',
        BROWSER: 'browser'
    };

    global.CosmicClimberPAL = global.CosmicClimberPAL || {};
    global.CosmicClimberPAL.PLATFORM_IDS = PLATFORM_IDS;
    global.CosmicClimberPAL.platforms = global.CosmicClimberPAL.platforms || {};

    global.CosmicClimberPAL.registerPlatform = function (id, impl) {
        if (!impl || !impl.keyMapping) {
            console.warn('[CosmicClimberPAL] registerPlatform: missing keyMapping for', id);
            return;
        }
        global.CosmicClimberPAL.platforms[id] = impl;
    };
})(typeof window !== 'undefined' ? window : this);
