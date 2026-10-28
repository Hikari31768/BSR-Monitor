import { useState } from "react";
import CopyKeyButton from "./CopyKeyButton";
import OpenBeatSaverLinkButton from "./OpenBeatSaverLinkButton";

interface MapKeyButtonsProps {
  mapKey: string;
}

enum ClickType {
  none,
  copy,
  openBeatSaverLink
}

export default function MapKeyButtons({ mapKey }: MapKeyButtonsProps) {
  const [clicked, setClicked] = useState(false);
  const [clickType, setClickType] = useState(ClickType.none);

  function clickText() {
    switch (clickType) {
      case ClickType.none: {
        return "";
      }
      case ClickType.copy: {
        return "!bsr 已复制";
      }
      case ClickType.openBeatSaverLink: {
        return "跳转到谱面网页";
      }
    }
  }

  function onButtonClick(clickType: ClickType) {
    setClickType(clickType);
    setClicked(true);
    setTimeout(() => {
      setClicked(false);
      setClickType(ClickType.none);
    }, 1200);
  }

  return (
    <div className="flex items-center gap-1 mt-1">
      <p className="text-xs text-bs-text-secondary">{mapKey}</p>
      {
        clicked ? <span
          className={`inline-flex items-center justify-center rounded h-5 px-2 text-xs font-semibold leading-none text-bs-accent-purple bg-bs-accent-purple/20 transition-opacity duration-100 border border-bs-accent-purple/40`}
        >
          {clickText()}
        </span> : (<>
          <CopyKeyButton mapKey={mapKey} onButtonClick={() => { onButtonClick(ClickType.copy) }} />
          <OpenBeatSaverLinkButton mapKey={mapKey} onButtonClick={() => { onButtonClick(ClickType.openBeatSaverLink) }} />
        </>)
      }
    </div>
  );
}
