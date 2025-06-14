import { useNavigate } from "react-router-dom";

import type { Gateway } from "@chirpstack/chirpstack-api-grpc-web/api/gateway_pb";
import { UpdateGatewayRequest } from "@chirpstack/chirpstack-api-grpc-web/api/gateway_pb";

import GatewayForm from "./GatewayForm";
import GatewayStore from "../../stores/GatewayStore";
import SessionStore from "../../stores/SessionStore";

interface IProps {
  gateway: Gateway;
}

function EditGateway(props: IProps) {
  const navigate = useNavigate();

  const onFinish = (obj: Gateway) => {
    const req = new UpdateGatewayRequest();
    req.setGateway(obj);

    GatewayStore.update(req, () => {
      navigate(`/tenants/${obj.getOrganizationId()}/gateways/${obj.getId()}`);
    });
  };

  const disabled = !(
    SessionStore.isAdmin() ||
    SessionStore.isTenantAdmin(props.gateway.getOrganizationId()) ||
    SessionStore.isTenantGatewayAdmin(props.gateway.getOrganizationId())
  );
  return <GatewayForm initialValues={props.gateway} onFinish={onFinish} disabled={disabled} update />;
}

export default EditGateway;
