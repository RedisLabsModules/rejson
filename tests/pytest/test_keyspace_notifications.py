import time

import redis
from RLTest import Env
import time

def assert_msg(env, msg, expected_type, expected_data):
    env.assertEqual(expected_type, msg['type']) 
    env.assertEqual(expected_data, msg['data']) 

def test_keyspace_set(env):
    with env.getClusterConnectionIfNeeded() as r:
        r.execute_command('config', 'set', 'notify-keyspace-events', 'KEA')

        pubsub = r.pubsub()
        pubsub.psubscribe('__key*')

        time.sleep(1)
        env.assertEqual('psubscribe', pubsub.get_message()['type']) 

        r.execute_command('JSON.SET', 'test_key', '$', '{"foo": "bar"}')
        assert_msg(env, pubsub.get_message(), 'pmessage', 'json.set')
        assert_msg(env, pubsub.get_message(), 'pmessage', 'test_key')

        env.assertEqual('OK', r.execute_command('JSON.SET', 'test_key', '$.foo', '"gogo"'))
        assert_msg(env, pubsub.get_message(), 'pmessage', 'json.set')
        assert_msg(env, pubsub.get_message(), 'pmessage', 'test_key')

        env.assertEqual(8, r.execute_command('JSON.STRAPPEND', 'test_key', '$.foo', '"toto"'))
        assert_msg(env, pubsub.get_message(), 'pmessage', 'json.strappend')
        assert_msg(env, pubsub.get_message(), 'pmessage', 'test_key')

        # Negative tests should not get an event
        env.assertEqual(None, r.execute_command('JSON.SET', 'test_key', '$.foo.a', '"nono"'))
        env.assertEqual(None, pubsub.get_message())       

        env.assertEqual(8, r.execute_command('JSON.STRLEN', 'test_key', '$.foo'))
        env.assertEqual(None, pubsub.get_message())       

        env.assertEqual('"gogototo"', r.execute_command('JSON.GET', 'test_key', '$.foo'))
        env.assertEqual(None, pubsub.get_message())       

        env.assertEqual(['"gogototo"', None], r.execute_command('JSON.MGET', 'test_key', 'test_key1', '$.foo'))
        env.assertEqual(None, pubsub.get_message())       

        env.assertEqual(['foo'], r.execute_command('JSON.OBJKEYS', 'test_key', '$'))
        env.assertEqual(None, pubsub.get_message())       

        env.assertEqual(1, r.execute_command('JSON.OBJLEN', 'test_key', '$'))
        env.assertEqual(None, pubsub.get_message())       

def test_keyspace_arr(env):
    with env.getClusterConnectionIfNeeded() as r:
        r.execute_command('config', 'set', 'notify-keyspace-events', 'KEA')

        pubsub = r.pubsub()
        pubsub.psubscribe('__key*')

        time.sleep(1)
        env.assertEqual('psubscribe', pubsub.get_message()['type']) 

        r.execute_command('JSON.SET', 'test_key_arr', '$', '{"foo": []}')
        assert_msg(env, pubsub.get_message(), 'pmessage', 'json.set')
        assert_msg(env, pubsub.get_message(), 'pmessage', 'test_key_arr')

        env.assertEqual(2, r.execute_command('JSON.ARRAPPEND', 'test_key_arr', '$.foo', '"gogo1"', '"gogo2"'))
        assert_msg(env, pubsub.get_message(), 'pmessage', 'json.arrappend')
        assert_msg(env, pubsub.get_message(), 'pmessage', 'test_key_arr')

        env.assertEqual(4, r.execute_command('JSON.ARRINSERT', 'test_key_arr', '$.foo', 1, '"gogo3"', '"gogo4"'))
        assert_msg(env, pubsub.get_message(), 'pmessage', 'json.arrinsert')
        assert_msg(env, pubsub.get_message(), 'pmessage', 'test_key_arr')

        env.assertEqual('"gogo3"', r.execute_command('JSON.ARRPOP', 'test_key_arr', '$.foo', 1))
        assert_msg(env, pubsub.get_message(), 'pmessage', 'json.arrpop')
        assert_msg(env, pubsub.get_message(), 'pmessage', 'test_key_arr')

        env.assertEqual(2, r.execute_command('JSON.ARRTRIM', 'test_key_arr', '$.foo', 0, 1))
        assert_msg(env, pubsub.get_message(), 'pmessage', 'json.arrtrim')
        assert_msg(env, pubsub.get_message(), 'pmessage', 'test_key_arr')

        # Negative tests should not get an event 
        env.assertEqual(0, r.execute_command('JSON.ARRINDEX', 'test_key_arr', '$.foo', '"gogo1"'))
        env.assertEqual(None, pubsub.get_message())   

        env.assertEqual(2, r.execute_command('JSON.ARRLEN', 'test_key_arr', '$.foo'))
        env.assertEqual(None, pubsub.get_message())   

        # TODO add more negative test for arr path not found

def test_keyspace_del(env):
    with env.getClusterConnectionIfNeeded() as r:
        r.execute_command('config', 'set', 'notify-keyspace-events', 'KEA')

        pubsub = r.pubsub()
        pubsub.psubscribe('__key*')

        time.sleep(1)
        env.assertEqual('psubscribe', pubsub.get_message()['type']) 

        r.execute_command('JSON.SET', 'test_key', '$', '{"foo": "bar", "foo2":"bar2", "foo3":"bar3"}')
        assert_msg(env, pubsub.get_message(), 'pmessage', 'json.set')
        assert_msg(env, pubsub.get_message(), 'pmessage', 'test_key')

        env.assertEqual(1, r.execute_command('JSON.DEL', 'test_key', '$.foo'))
        assert_msg(env, pubsub.get_message(), 'pmessage', 'json.del')
        assert_msg(env, pubsub.get_message(), 'pmessage', 'test_key')

        env.assertEqual(1, r.execute_command('JSON.FORGET', 'test_key', '$.foo3'))
        assert_msg(env, pubsub.get_message(), 'pmessage', 'json.del')
        assert_msg(env, pubsub.get_message(), 'pmessage', 'test_key')

        env.assertEqual(0, r.execute_command('JSON.DEL', 'test_key', '$.foo'))
        env.assertEqual(None, pubsub.get_message())      

def test_keyspace_num(env):
    with env.getClusterConnectionIfNeeded() as r:
        r.execute_command('config', 'set', 'notify-keyspace-events', 'KEA')

        pubsub = r.pubsub()
        pubsub.psubscribe('__key*')

        time.sleep(1)
        env.assertEqual('psubscribe', pubsub.get_message()['type']) 

        r.execute_command('JSON.SET', 'test_key', '$', '{"foo": 1}')
        assert_msg(env, pubsub.get_message(), 'pmessage', 'json.set')
        assert_msg(env, pubsub.get_message(), 'pmessage', 'test_key')

        env.assertEqual('4', r.execute_command('JSON.NUMINCRBY', 'test_key', '$.foo', 3))
        assert_msg(env, pubsub.get_message(), 'pmessage', 'json.numincrby')
        assert_msg(env, pubsub.get_message(), 'pmessage', 'test_key')

        env.assertEqual('12', r.execute_command('JSON.NUMMULTBY', 'test_key', '$.foo', 3))
        assert_msg(env, pubsub.get_message(), 'pmessage', 'json.nummultby')
        assert_msg(env, pubsub.get_message(), 'pmessage', 'test_key')

        # TODO add negative test for number
