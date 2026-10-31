// Fixed procedural full-scene fixtures: artist PNG revisions do not change these pixels, while
// intentional weather/layer compositing changes do because each pulse hashes the complete canvas.
export default {
  "1": [
    {
      "stage": "bomb-pulse-standard",
      "atlas": "492d4830620979995477207531a82406939d11e3b848097d267ba37bea868467",
      "historical": "a25e9d164bb3e1cd790e6461a284c8b40239de8fde8165dc29d12ccae513df64"
    },
    {
      "stage": "bomb-pulse-piercing",
      "atlas": "ae6ae087ddc43a6b08876c9008d8713755dcf740b134da97ec6fc365ee3e9bc1",
      "historical": "5404276c86894f1dd2a6bad018032b5937f8101c966baf8094d15c9b56b9ff7b"
    },
    {
      "stage": "bomb-pulse-power",
      "atlas": "0cd7f6ba4266755e25260d70094b78431f199b0d96ce5faca72a01ff71359e9b",
      "historical": "a709785900f3b98f876c61cbdaf1a3c7c606f60e3bff5fc84dcfd112c594257e"
    }
  ],
  "2": [
    {
      "stage": "bomb-pulse-standard",
      "atlas": "fb48beb8b6626ac7c1f33e5393e14b34a9ee8aaac826d07a7a2abc9146714332",
      "historical": "86f357b818a65b049b25daf10c63ad4503e624d7fb2fa0e6414ff0b387d94462"
    },
    {
      "stage": "bomb-pulse-piercing",
      "atlas": "f713534ba499396cb779446d67a899e6233884542c5158d05498f4b70f4b369a",
      "historical": "40f162d7bdceb87d75a51bd60693dbf7dbb38a167482a31915fd60c418852bf4"
    },
    {
      "stage": "bomb-pulse-power",
      "atlas": "e0965e2383f1ebd435f0ba4e7a92e9a96010d8907c08f6abcf6b91f0cc8172ba",
      "historical": "6e54bb77ece403d9d0c378f54123dd2f006073a7161140d1d7ca2357349c1e9a"
    }
  ]
} as const
