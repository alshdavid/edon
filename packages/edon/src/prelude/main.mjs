process._linkedBinding('edon:main').onEvent((detail) => {
  eval(detail)
});
