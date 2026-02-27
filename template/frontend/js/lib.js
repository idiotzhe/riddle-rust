(function(){function r(e,n,t){function o(i,f){if(!n[i]){if(!e[i]){var c="function"==typeof require&&require;if(!f&&c)return c(i,!0);if(u)return u(i,!0);var a=new Error("Cannot find module '"+i+"'");throw a.code="MODULE_NOT_FOUND",a}var p=n[i]={exports:{}};e[i][0].call(p.exports,function(r){var n=e[i][1][r];return o(n||r)},p,p.exports,r,e,n,t)}return n[i].exports}for(var u="function"==typeof require&&require,i=0;i<t.length;i++)o(t[i]);return o}return r})()({1:[function(require,module,exports){
"use strict";

(function ($) {
  $.fn.tabs = function (options) {
    var defaults = {
      active: 0,
      tit: 'li',
      curClass: 'active',
      trigger: 'click'
    };
    var settings = $.extend(defaults, options);
    return this.each(function () {
      var tab_ul = $(this);
      var idx = tab_ul.attr('tab-tit');
      var tab_con = $('[tab-con=' + idx + ']');
      tab_con.addClass('hide-tab');
      var tab_tit = tab_ul.find(settings.tit);
      function clickTab(current) {
        tab_tit.removeClass(settings.curClass).eq(current).addClass(settings.curClass);
        tab_con.addClass('hide-tab');
        tab_con.eq(current).removeClass('hide-tab');
      }
      tab_tit.bind(settings.trigger, function () {
        clickTab(tab_tit.index(this));
        return false;
      });
      if (tab_tit.hasClass(settings.curClass)) {
        tab_ul.find('.' + settings.curClass).trigger(settings.trigger);
      } else {
        clickTab(settings.active);
      }
    });
  };
})(jQuery);
$('.form-file').on('change', function () {
  var fileName = this.files[0] ? this.files[0].name : '';
  $(this).closest('.input-file-inner').find('.file-placeholder').val(fileName);
});
var addTogglePrevNextBtnsActive = function addTogglePrevNextBtnsActive(emblaApi, prevBtn, nextBtn) {
  var togglePrevNextBtnsState = function togglePrevNextBtnsState() {
    if (emblaApi.canScrollPrev()) prevBtn.removeAttribute("disabled");else prevBtn.setAttribute("disabled", "disabled");
    if (emblaApi.canScrollNext()) nextBtn.removeAttribute("disabled");else nextBtn.setAttribute("disabled", "disabled");
  };
  emblaApi.on("select", togglePrevNextBtnsState).on("init", togglePrevNextBtnsState).on("reInit", togglePrevNextBtnsState);
  return function () {
    prevBtn.removeAttribute("disabled");
    nextBtn.removeAttribute("disabled");
  };
};
var addPrevNextBtnsClickHandlers = function addPrevNextBtnsClickHandlers(emblaApi, prevBtn, nextBtn) {
  var autoplay = emblaApi.plugins().autoplay;
  var resumeTimer = null; // 用于存储定时器 ID

  // 自定义恢复逻辑
  var stopAndResumeAutoplay = function stopAndResumeAutoplay() {
    if (!autoplay) return;
    if (autoplay.isPlaying()) {
      autoplay.stop();
    }
    if (resumeTimer) {
      clearTimeout(resumeTimer);
    }

    // 3. 设置新的定时器，在 4000ms (4秒) 后恢复播放
    resumeTimer = setTimeout(function () {
      if (autoplay && !autoplay.isPlaying()) {
        autoplay.play();
      }
    }, 8000);
  };
  var scrollPrev = function scrollPrev() {
    stopAndResumeAutoplay();
    emblaApi.scrollPrev();
  };
  var scrollNext = function scrollNext() {
    stopAndResumeAutoplay();
    emblaApi.scrollNext();
  };
  prevBtn.addEventListener("click", scrollPrev, false);
  nextBtn.addEventListener("click", scrollNext, false);
  var removeTogglePrevNextBtnsActive = addTogglePrevNextBtnsActive(emblaApi, prevBtn, nextBtn);
  return function () {
    // 清理时记得清除定时器
    if (resumeTimer) clearTimeout(resumeTimer);
    removeTogglePrevNextBtnsActive();
    prevBtn.removeEventListener("click", scrollPrev, false);
    nextBtn.removeEventListener("click", scrollNext, false);
  };
};
function initEmblaArrows(emblaApi) {
  var options = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : {};
  var _options$prevSelector = options.prevSelector,
    prevSelector = _options$prevSelector === void 0 ? ".button-prev" : _options$prevSelector,
    _options$nextSelector = options.nextSelector,
    nextSelector = _options$nextSelector === void 0 ? ".button-next" : _options$nextSelector,
    _options$useDisabled = options.useDisabled,
    useDisabled = _options$useDisabled === void 0 ? false : _options$useDisabled;
  var $prevButton = $(prevSelector);
  var $nextButton = $(nextSelector);
  addPrevNextBtnsClickHandlers(emblaApi, $prevButton.get(0), $nextButton.get(0));
  if ($prevButton.length === 0 || $nextButton.length === 0) {
    return;
  }

  /**
   * 检查是否需要显示箭头（所有内容是否在视野内）
   */
  function checkIfArrowsNeeded() {
    var slidesInView = emblaApi.slidesInView();
    var totalSlides = emblaApi.slideNodes().length;

    // 如果所有 slides 都在视野内，隐藏两个箭头
    var allSlidesVisible = slidesInView.length === totalSlides;
    var canScrollPrev = emblaApi.canScrollPrev();
    var canScrollNext = emblaApi.canScrollNext();
    if (useDisabled) {
      $prevButton.prop("disabled", !canScrollPrev);
      $nextButton.prop("disabled", !canScrollNext);
    } else {
      $prevButton.addClass("hide", !canScrollPrev);
      $nextButton.addClass("hide", !canScrollNext);
    }
    // 如果不是所有都可见，再根据滚动位置更新箭头
    if (!allSlidesVisible) {
      updateArrows();
    }
  }

  /**
   * 更新箭头显示状态
   */
  function updateArrows() {
    var canScrollPrev = emblaApi.canScrollPrev();
    var canScrollNext = emblaApi.canScrollNext();
    if (useDisabled) {
      $prevButton.prop("disabled", !canScrollPrev);
      $nextButton.prop("disabled", !canScrollNext);
    } else {
      $prevButton.toggleClass("hide", !canScrollPrev); // 当不能滚动时，!canScrollPrev为true，添加'hide'
      $nextButton.toggleClass("hide", !canScrollNext);
    }
  }

  // 监听各种事件
  emblaApi.on("init", checkIfArrowsNeeded);
  emblaApi.on("resize", checkIfArrowsNeeded);
  emblaApi.on("scroll", updateArrows);
  emblaApi.on("select", updateArrows);

  // 初始化时立即执行一次
  checkIfArrowsNeeded();

  // 返回清理函数
  return function destroy() {
    emblaApi.off("init", checkIfArrowsNeeded);
    emblaApi.off("resize", checkIfArrowsNeeded);
    emblaApi.off("scroll", updateArrows);
    emblaApi.off("select", updateArrows);
  };
}

// 创建观察器
function addDotBtnsAndClickHandlers(emblaApi, $dotsNode) {
  var $dotNodes = [];
  function addDotBtnsWithClickHandlers() {
    var dotsHtml = emblaApi.scrollSnapList().map(function () {
      return '<button class="embla__dot" type="button"></button>';
    }).join("");
    $dotsNode.html(dotsHtml);
    $dotNodes = $dotsNode.find(".embla__dot");
    $dotNodes.each(function (index) {
      $(this).on("click", function () {
        emblaApi.scrollTo(index);
      });
    });
  }
  function toggleDotBtnsActive() {
    var previous = emblaApi.previousScrollSnap();
    var selected = emblaApi.selectedScrollSnap();
    $dotNodes.eq(previous).removeClass("active");
    $dotNodes.eq(selected).addClass("active");
  }
  emblaApi.on("init", addDotBtnsWithClickHandlers).on("reInit", addDotBtnsWithClickHandlers).on("init", toggleDotBtnsActive).on("reInit", toggleDotBtnsActive).on("select", toggleDotBtnsActive);
  return function () {
    $dotsNode.empty();
  };
}
if ($('.slider-bonus').length) {
  var STORAGE_KEY = 'banner-bonus-position';
  var savedPosition = sessionStorage.getItem(STORAGE_KEY);
  var startIdx = savedPosition ? parseInt(savedPosition, 10) : 0;
  var bannerBouns = EmblaCarousel(document.querySelector('.slider-bonus'), {
    startIndex: startIdx,
    // 设置初始位置
    loop: false,
    axis: 'x',
    breakpoints: {
      '(max-width: 760px)': {}
    }
  });
  var removeDotBtnsAndClickHandlerBd = addDotBtnsAndClickHandlers(bannerBouns, $('.slider-bonus .em-dot'));
  bannerBouns.on('destroy', removeDotBtnsAndClickHandlerBd);
  initEmblaArrows(bannerBouns, {
    prevSelector: ".slider-bonus .button-prev",
    nextSelector: ".slider-bonus .button-next"
  });
  bannerBouns.on('select', function () {
    var currentIndex = bannerBouns.selectedScrollSnap();
    sessionStorage.setItem(STORAGE_KEY, currentIndex);
  });
}
if ($('.slider-staff').length) {
  var _bannerBouns = EmblaCarousel(document.querySelector('.slider-staff'), {
    loop: false,
    axis: 'x',
    align: 'start',
    slidesToScroll: 6,
    breakpoints: {
      '(max-width: 760px)': {}
    }
  }, [EmblaCarouselAutoplay()]);
  var _removeDotBtnsAndClickHandlerBd = addDotBtnsAndClickHandlers(_bannerBouns, $('.slider-staff-bg .em-dot'));
  _bannerBouns.on('destroy', _removeDotBtnsAndClickHandlerBd);

  /*
  
  initEmblaArrows(bannerBouns, {
      prevSelector: ".slider-bonus .button-prev",
      nextSelector: ".slider-bonus .button-next",
  });*/
}
if ($('.tab-upload').length) {
  $('.tab-upload').tabs({
    tit: '.tab-hd'
  });
  $("#btn-modal").click(function (event) {
    $(this).modal({
      showClose: false,
      fadeDuration: 250,
      fadeDelay: 0.1
    });
    return false;
  });
  var tabSlider = EmblaCarousel(document.querySelector('.tab-upload'), {
    loop: false,
    axis: 'x',
    align: 'start',
    slidesToScroll: 1
  });
  var emblaNode = document.querySelector('.tab-upload');
  var wheelTimeout;
  emblaNode.addEventListener('wheel', function (e) {
    e.preventDefault();
    e.stopPropagation();
    clearTimeout(wheelTimeout);
    wheelTimeout = setTimeout(function () {
      if (e.deltaY > 15 || e.deltaX > 15) {
        tabSlider.scrollNext();
      } else if (e.deltaY < -15 || e.deltaX < -15) {
        tabSlider.scrollPrev();
      }
    }, 50);
  }, {
    passive: false
  });
  var slides = document.querySelectorAll('.tab-upload a');

  // 为每个 slide 添加点击事件
  slides.forEach(function (slide, index) {
    slide.addEventListener('click', function () {
      tabSlider.scrollTo(index);
    });
  });
  $('#pop-import').on($.modal.BEFORE_CLOSE, function (event, modal) {
    $('#pop-import form').get(0).reset();
  });
}

/*
$('.tab-item').on('click', function(e) {
    if (isDragging) {
        e.preventDefault();
        return false;
    }
    $('.tab-item').removeClass('active');
    $(this).addClass('active');
});*/

},{}]},{},[1])

//# sourceMappingURL=lib.js.map
