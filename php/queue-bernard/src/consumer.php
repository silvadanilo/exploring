<?php
include 'vendor/autoload.php';

use Bernard\Consumer;
use Bernard\Driver\FlatFileDriver;
use Bernard\Middleware;
use Bernard\Message;
use Bernard\QueueFactory\PersistentFactory;
use Bernard\Router\SimpleRouter;
use Bernard\Serializer\SimpleSerializer;

// .. create driver and a queuefactory
// NewsletterMessageHandler is a pseudo service object that responds to
// sendNewsletter.
$driver = new FlatFileDriver('/tmp/messages');
$queueFactory = new PersistentFactory($driver, new SimpleSerializer());

$router = new SimpleRouter();
$router->add('SendNewsletter', new NewsletterMessageHandler);

// Bernard also comes with a router for Pimple (Silex) which allows you
// to use service ids and have your service object lazy loader.
//
// $router = new \Bernard\Router\PimpleAwareRouter($pimple);
// $router->add('SendNewsletter', 'my.service.id');
//
// Symfony DependencyInjection component is also supported.
//
// $router = new \Bernard\Router\ContainerAwareRouter($container);
// $router->add('SendNewsletter', 'my.service.id');
//
//
$eventDispatcher = new Middleware\MiddlewareBuilder;

// Create a Consumer and start the loop.
$consumer = new Consumer($router, $eventDispatcher);

// The second argument is optional and is an array
// of options. Currently only ``max-runtime`` is supported which specifies the max runtime
// in seconds.
$consumer->consume($queueFactory->create('send-newsletter'), array(
    'max-runtime' => 100,
));

class NewsletterMessageHandler
{
    public function sendNewsletter(Message $foo) {
        var_export($foo->newsletterId);
    }
}
