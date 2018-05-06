<?php
require_once 'bootstrap.php';
/** @var \Enqueue\Redis\RedisContext $psrContext */

$fooTopic = $psrContext->createTopic('aTopic');
$consumer = $psrContext->createConsumer($fooTopic);
$message = $consumer->receive();
var_export($message);


$fooQueue = $psrContext->createQueue('aQueue');
$consumer = $psrContext->createConsumer($fooQueue);
$message = $consumer->receive();
var_export($message);
