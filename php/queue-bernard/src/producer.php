<?php
include 'vendor/autoload.php';

use Bernard\Driver\FlatFileDriver;
use Bernard\Message\DefaultMessage;
use Bernard\Middleware;
use Bernard\Producer;
use Bernard\QueueFactory\PersistentFactory;
use Bernard\Serializer\SimpleSerializer;

$driver = new FlatFileDriver('/tmp/messages');
$factory = new PersistentFactory($driver, new SimpleSerializer());
$producerMiddleware = new Middleware\MiddlewareBuilder;
$producer = new Producer($factory, $producerMiddleware);

$message = new DefaultMessage('SendNewsletter', array(
    'newsletterId' => 12,
));

$producer->produce($message);
