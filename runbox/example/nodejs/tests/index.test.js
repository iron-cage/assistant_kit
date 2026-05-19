const { test } = require('node:test');
const assert = require('node:assert/strict');
const { add } = require('../src/index.js');

test('add positive numbers', () => {
  assert.equal(add(2, 3), 5);
});

test('add negative numbers', () => {
  assert.equal(add(-1, 1), 0);
});

test('add zeros', () => {
  assert.equal(add(0, 0), 0);
});
