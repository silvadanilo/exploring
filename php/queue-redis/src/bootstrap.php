<?php
include __DIR__ . '/../vendor/autoload.php';

use Enqueue\Redis\RedisConnectionFactory;


/* // connect to Redis at example.com port 1000 using phpredis extension */
$factory = new RedisConnectionFactory([
    'host' => 'localhost',
    'port' => 32768,
    'vendor' => 'phpredis',
]);
$psrContext = $factory->createContext();

// if you have enqueue/enqueue library installed you can use a function from there to create the context
/* $psrContext = \Enqueue\dsn_to_context('redis:'); */
