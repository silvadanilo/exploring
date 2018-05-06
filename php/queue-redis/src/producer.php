<?php
require_once 'bootstrap.php';
/** @var \Enqueue\Redis\RedisContext $psrContext */

$fooTopic = $psrContext->createTopic('aTopic');
$message = $psrContext->createMessage('Hello world!');
$psrContext->createProducer()->send($fooTopic, $message);


$fooQueue = $psrContext->createQueue('aQueue');
$message = $psrContext->createMessage('Hello world!');
$psrContext->createProducer()->send($fooQueue, $message);
