(function(){function r(e,n,t){function o(i,f){if(!n[i]){if(!e[i]){var c="function"==typeof require&&require;if(!f&&c)return c(i,!0);if(u)return u(i,!0);var a=new Error("Cannot find module '"+i+"'");throw a.code="MODULE_NOT_FOUND",a}var p=n[i]={exports:{}};e[i][0].call(p.exports,function(r){var n=e[i][1][r];return o(n||r)},p,p.exports,r,e,n,t)}return n[i].exports}for(var u="function"==typeof require&&require,i=0;i<t.length;i++)o(t[i]);return o}return r})()({1:[function(require,module,exports){
"use strict";

$('.btn-history').on('click', function (e) {
  e.preventDefault();
  e.stopPropagation();
  $('#modal-history').modal({
    showClose: false,
    fadeDuration: 250,
    fadeDelay: 0.1
  });
  return false;
});
$('#modal-history').on($.modal.BEFORE_OPEN, function (event, modal) {
  window.parent.postMessage({
    type: 'popOpen',
    timestamp: Date.now()
  }, '*');
});
$('#modal-history').on($.modal.BEFORE_CLOSE, function (event, modal) {
  window.parent.postMessage({
    type: 'popClose',
    timestamp: Date.now()
  }, '*');
});
$(document).ready(function () {
  var $fileInput = $('#fileInput');
  var $imgPreview = $('#imgPreview');
  var $avtPreview = $('#avt-preview');
  var $uploadLabel = $('#uploadLabel');
  var $deleteBtn = $('#deleteBtn'); // 监听文件选择

  $fileInput.on('change', function (e) {
    var file = e.target.files[0];

    if (file && file.type.startsWith('image/')) {
      var reader = new FileReader();

      reader.onload = function (e) {
        $imgPreview.attr('src', e.target.result);
        $avtPreview.addClass('show');
        $uploadLabel.text('重新上传');
        $deleteBtn.fadeIn(200);
      };

      reader.readAsDataURL(file);
    }
  }); // 监听删除按钮

  $deleteBtn.on('click', function () {
    $fileInput.val('');
    $imgPreview.attr('src', '').hide();
    $uploadLabel.text('选择图片'); // 文字变回 "选择图片"

    $(this).fadeOut(200); // 隐藏删除按钮
  });
});

},{}]},{},[1]);

//# sourceMappingURL=lib.js.map
