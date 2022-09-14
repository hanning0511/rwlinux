import {
  Row,
  Col,
  Form,
  Tooltip,
  Input,
  Radio,
  Tag,
  Modal,
  message,
} from "antd";
import { useState, useEffect, useCallback } from "react";
import "./hexeditor.css";

export const useDevmem = (initStatus) => {
  const [bytes, setBytes] = useState(initStatus);
  const readBytes = useCallback((addr, offset, length) => {
    const url = encodeURI(`${addr}/devmem?offset=${offset}&length=${length}`);
    fetch(url).then((response) => {
      if (response.status !== 200) {
        message.log("fail to read /dev/mem on remote host!");
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

const Devmem = (props) => {
  const { connection } = props;
  const { bytes, readBytes } = useDevmem(null);
  const [cellType, setCellType] = useState("b");
  const [address, setAddress] = useState(0);
  const [selected, setSelected] = useState(0);
  const [selectedValue, setSelectedValue] = useState(null);
  const [pageStart, setPageStart] = useState(0);
  const [editVisible, setEditVisible] = useState(false);
  const hostAddr = `http://${connection}`;
  const rowSize = 16;
  const colSize = 16;

  useEffect(() => {
    let addr = parseInt("0x" + address, 16);
    let remainder = addr % (rowSize * colSize);
    let pageStart = addr - remainder;

    setPageStart(pageStart);
    setSelected(remainder);
    readBytes(hostAddr, pageStart, rowSize * colSize);
  }, [props, address]);

  useEffect(() => {
    if (bytes === null) {
      return;
    }

    let cellLength;
    switch (cellType) {
      case "byte":
        cellLength = 1;
        break;
      case "w":
        cellLength = 2;
        break;
      case "d":
        cellLength = 4;
        break;
      case "q":
        cellLength = 8;
        break;
      default:
        cellLength = 1;
    }

    setSelectedValue(bytes.slice(selected, selected + cellLength));

    // Create a table
    const table = document.createElement("table");
    table.className = "editor-table";

    for (let i = 0; i < rowSize * colSize; i += rowSize) {
      const end = i + rowSize;
      let row = [];

      if (end < bytes.length) {
        row = bytes.slice(i, end);
      } else {
        row = bytes.slice(i, bytes.length);
      }

      const tr = document.createElement("tr");
      tr.className = "editor-tr";

      for (let j = 0; j < row.length; j += cellLength) {
        const td = document.createElement("td");
        const cellContent = document.createElement("tt");

        td.className = "editor-td";
        td.addEventListener("mouseover", (event) => {
          if (event.target.key !== undefined) {
            setSelected(event.target.key);
          }
        });
        td.addEventListener("dblclick", (event) => {
          setEditVisible(true);
          if (event.target.key !== undefined) {
            setEditVisible(true);
          }
        });

        if (i + j === selected) {
          td.style.backgroundColor = "green";
          td.style.color = "white";
        }

        cellContent.innerHTML = "";
        row
          .slice(j, j + cellLength)
          .reverse()
          .forEach((element) => {
            cellContent.innerHTML += element.toString(16).padStart(2, "0");
          });
        cellContent.key = i + j;

        td.appendChild(cellContent);
        tr.appendChild(td);
      }

      table.appendChild(tr);
    }

    const tb = document.getElementById("hex-table");

    tb.innerHTML = "";
    tb.appendChild(table);
  }, [bytes, selected, cellType]);

  return (
    <div style={{ padding: "5vh 2vw" }}>
      <Row type="flex" justify="center" style={{ textAlign: "center" }}>
        <Col style={{ marginLeft: "1vw" }}>
          <Radio.Group
            value={cellType}
            options={[
              {
                label: "Byte(8)",
                value: "b",
              },
              {
                label: "Word(16)",
                value: "w",
              },
              {
                label: "Double Word(32)",
                value: "d",
              },
              {
                label: "Quad Word(64)",
                value: "q",
              },
            ]}
            optionType="button"
            onChange={(e) => setCellType(e.target.value)}
          />
        </Col>
      </Row>

      <br />

      <Row type="flex" justify="center">
        <Col>
          <div id="hex-table"></div>
        </Col>
        <Col>
          <div id="ascii-table"></div>
        </Col>
      </Row>

      <br />

      <Row type="flex" justify="center">
        <Col>
          <Form>
            <Form.Item label="Address(Hex)">
              <Tooltip title="press Enter to jump">
                <Input
                  size="small"
                  style={{ width: "200px", textAlign: "center" }}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") {
                      setAddress(e.target.value);
                    }
                  }}
                />
              </Tooltip>
            </Form.Item>
          </Form>
        </Col>

        <Col style={{ marginLeft: "10px", paddingTop: "4px" }}>
          <Tag size="large">
            Offset: {`0x${(pageStart + selected).toString(16)}`}
          </Tag>
        </Col>
      </Row>

      {bytes !== null && (
        <Modal
          closable={false}
          visible={editVisible}
          onCancel={() => setEditVisible(false)}
          footer={null}
          width="50vw"
        >
          <Row type="flex" justify="end">
            <Radio.Group
              defaultValue={"byte"}
              options={[
                {
                  label: <tt>BIT</tt>,
                  value: "bit",
                },
                {
                  label: <tt>HEX</tt>,
                  value: "hex",
                },
              ]}
              optionType="button"
            />
          </Row>
          <Row type="flex" justify="center"></Row>
        </Modal>
      )}
    </div>
  );
};

export default Devmem;
