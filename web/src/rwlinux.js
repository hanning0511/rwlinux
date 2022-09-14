import { useEffect, useState } from "react";
import { useDevmem } from "./hook";
import { Row, Col, Form, Input } from "antd";
import HexEditor from "./hexeditor";
import "./rwlinux.css";

const Devmem = (props) => {
  const { bytes, readBytes } = useDevmem(null);
  const [address, setAddress] = useState(0);
  const [pageOffset, setPageOffset] = useState(0);

  useEffect(() => {
    const addr = `http://${props.ip}:${props.port}`;
    console.log(address);
    readBytes(addr, parseInt("0x" + address, 16), 256);
  }, [props, address]);

  useEffect(() => {
    console.log(bytes);
  }, [bytes]);

  return (
    <div style={{ margin: "20vh 2vw" }}>
      <Row type="flex" justify="center">
        {bytes !== null && (
          <HexEditor key={address} offset={pageOffset} data={bytes} />
        )}
      </Row>

      <br />

      <Row type="flex" justify="center" style={{ textAlign: "center" }}>
        <Form>
          <Form.Item label="Address">
            <Input
              prefix="0x"
              style={{ width: "20vw" }}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  setAddress(e.target.value);
                }
              }}
            />
          </Form.Item>
        </Form>
      </Row>
    </div>
  );
};

const RWHost = (props) => {
  return <Devmem ip={props.ip} port={props.port} />;
};

const RWLinux = () => {
  return (
    <div>
      <RWHost ip="ninghan-desk.sh.intel.com" port="8000" />
    </div>
  );
};

export default RWLinux;
