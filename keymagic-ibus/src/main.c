#include <ibus.h>
#include <glib.h>
#include <locale.h>
#include <stdlib.h>
#include "engine.h"

/* Command line options */
static gboolean ibus = FALSE;
static gboolean verbose = FALSE;

static const GOptionEntry entries[] = {
    { "ibus", 'i', 0, G_OPTION_ARG_NONE, &ibus, "Component is executed by IBus", NULL },
    { "verbose", 'v', 0, G_OPTION_ARG_NONE, &verbose, "Verbose mode", NULL },
    { NULL },
};

/* IBus component factory */
static IBusFactory *factory = NULL;

/**
 * Initialize IBus component
 */
static void
init_ibus_component(IBusBus *bus)
{
    IBusComponent *component;
    IBusEngineDesc *engine_desc;
    
    /* Create IBus component */
    component = ibus_component_new(
        "org.freedesktop.IBus.KeyMagic3",   /* name */
        "KeyMagic 3 Input Method",           /* description */
        VERSION,                            /* version */
        "GPL-3.0",                          /* license */
        "Thant Thet Khin Zaw",              /* author */
        "https://github.com/thantthet/keymagic-3", /* homepage */
        "",                                 /* command line */
        "keymagic3"                         /* textdomain */
    );
    
    /* Create engine description */
    engine_desc = ibus_engine_desc_new(
        ibus ? "keymagic3" : "keymagic3-debug", /* name */
        "KeyMagic 3",                           /* longname */
        "KeyMagic 3 Input Method for Myanmar and other languages", /* description */
        "my",                               /* language */
        "GPL-3.0",                          /* license */
        "Thant Thet Khin Zaw",              /* author */
        "keymagic3.png",                    /* icon */
        "us"                                /* layout */
    );
    
    /* Add engine to component */
    ibus_component_add_engine(component, engine_desc);
    
    /* Create factory */
    factory = ibus_factory_new(ibus_bus_get_connection(bus));
    
    if (ibus) {
        /* Running under IBus - use standard name */
        ibus_factory_add_engine(factory, "keymagic3", KEYMAGIC_TYPE_ENGINE);
        ibus_bus_request_name(bus, "org.freedesktop.IBus.KeyMagic3", 0);
    } else {
        /* Debug mode - use debug name and register component */
        ibus_factory_add_engine(factory, "keymagic3-debug", KEYMAGIC_TYPE_ENGINE);
        ibus_bus_register_component(bus, component);
    }
    
    g_debug("IBus component initialized successfully (mode: %s)", 
            ibus ? "production" : "debug");
}

/**
 * Main entry point
 */
int
main(int argc, char *argv[])
{
    GError *error = NULL;
    GOptionContext *context;
    GMainLoop *main_loop;
    
    /* Set locale */
    setlocale(LC_ALL, "");
    
    /* Parse command line options */
    context = g_option_context_new("- KeyMagic 3 IBus Engine");
    g_option_context_add_main_entries(context, entries, "keymagic3");
    
    if (!g_option_context_parse(context, &argc, &argv, &error)) {
        g_printerr("Option parsing failed: %s\n", error->message);
        g_error_free(error);
        g_option_context_free(context);
        return EXIT_FAILURE;
    }
    
    g_option_context_free(context);
    
    /* Initialize GLib - g_thread_init is no longer needed in modern GLib */
    
    /* Set up logging */
    if (verbose) {
        g_log_set_handler(NULL, 
                         G_LOG_LEVEL_DEBUG | G_LOG_LEVEL_INFO | 
                         G_LOG_LEVEL_MESSAGE | G_LOG_LEVEL_WARNING | 
                         G_LOG_LEVEL_CRITICAL | G_LOG_LEVEL_ERROR,
                         g_log_default_handler, NULL);
    }
    
    /* Initialize IBus */
    ibus_init();
    
    /* Get IBus bus */
    IBusBus *bus = ibus_bus_new();
    if (!ibus_bus_is_connected(bus)) {
        g_printerr("Cannot connect to IBus daemon\n");
        g_object_unref(bus);
        return EXIT_FAILURE;
    }
    
    /* Initialize IBus component */
    init_ibus_component(bus);
    
    if (!ibus) {
        g_message("Running in debug mode (without --ibus flag)");
        g_message("You can test the engine using:");
        g_message("  ibus engine keymagic3-debug");
    }
    
    g_debug("Starting KeyMagic 3 IBus engine main loop");
    
    /* Run main loop */
    main_loop = g_main_loop_new(NULL, FALSE);
    g_main_loop_run(main_loop);
    
    /* Cleanup */
    g_main_loop_unref(main_loop);
    
    if (factory) {
        g_object_unref(factory);
    }
    
    g_debug("KeyMagic 3 IBus engine shutting down");
    
    return EXIT_SUCCESS;
}