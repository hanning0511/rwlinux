import { message } from "antd";
import { useCallback, useState } from "react";

export const usePCIDevices = (initStatus) => {
  const [devices, setDevices] = useState(initStatus);
  const getDevices = useCallback((addr) => {
    const url = encodeURI(`${addr}/pci/devices`);
    fetch(url).then((response) => {
      if (response.status !== 200) {
        message.log("fail to get pci device list");
        return;
      }
      response.json().then((data) => setDevices(data));
    });
  }, []);
  return { devices, getDevices };
};

export const usePciDeviceConfig = (initStatus) => {
  const [configs, setConfigs] = useState(initStatus);
  const getConfigs = useCallback((addr, domain, bus, dev, func) => {
    const url = encodeURI(
      `${addr}/pci/device/config?domain=${domain}&bus=${bus}&device=${dev}&function=${func}`
    );
    fetch(url).then((response) => {
      if (response.status !== 200) {
        message.error("fail to get pci device configuration data");
        return;
      }
      response.arrayBuffer().then((data) => {
        let converted = new Uint8Array(data);
        setConfigs(converted);
      });
    });
  }, []);
  return { configs, getConfigs };
};

export const useDevmem = (initStatus) => {
  const [bytes, setBytes] = useState(initStatus);
  const readBytes = useCallback((addr, offset, length) => {
    const url = encodeURI(`${addr}/devmem?offset=${offset}&length=${length}`);
    fetch(url).then((response) => {
      if (response.status !== 200) {
        console.log("fail to read /dev/mem on remote host!");
        return;
      }
      response.arrayBuffer().then((data) => {
        let converted = new Uint8Array(data);
        setBytes(converted);
      });
    });
  }, []);
  return { bytes, readBytes };
};
